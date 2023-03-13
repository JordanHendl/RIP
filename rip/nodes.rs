extern crate runa;
extern crate json;
use std::{collections::{HashMap, HashSet, hash_map::Entry}, hash::Hash, cell::RefCell, rc::Rc, io::Read};
use protobuf::Message;
use runa::{*, gpu::ImageView};
use json::*;
use crate::common::data_bus::DataBus;

use super::common;
use super::network;

mod finishers;
mod starters;
mod processors;
mod node_parser;

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
pub use message::*;

pub trait RipNode {
  fn name(&self) -> String;
  fn node_type(&self) -> String;
  fn assign(& mut self, view: &gpu::ImageView);
  fn input(& mut self, image: &gpu::ImageView); 
  fn execute(& mut self, cmd: & mut gpu::CommandList);
  fn post_execute(& mut self, _cmd: & mut gpu::CommandList) {}
  fn receive_message(& mut self, _cmd: & mut gpu::CommandList, _message: &message::Request) -> message::Response {
    let mut r = message::Response::new();
    let mut none_response = response::NoneResponse::new();
    none_response.msg = "Node has not implemented network responses!".to_string();
    r.set_none_response(none_response);
    return r;
  }
}

// Information needed for any node to create itself.
pub struct NodeCreateInfo {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  name: String,
}

pub struct Pipeline {
  node_name_map: HashMap<String, usize>,
  network: network::Manager,
  register: gpu::EventSubscriber,
  images: Vec<gpu::Image>,
  views: Vec<gpu::ImageView>,
  interface: Rc<RefCell<gpu::GPUInterface>>,
  nodes: Vec<Box<dyn RipNode + Send>>,
  execution_order: Vec<u32>,
  edges: HashMap<u32, Vec<u32>>,
  cmds: Vec<gpu::CommandList>,
  first: bool,
  should_run: Rc<RefCell<bool>>,
  json_file: String,
  json_configuration: String,
  json_timestamp: Option<std::time::SystemTime>,
}

impl Pipeline {

  pub fn parse_json(& mut self, path: &str) {
    self.nodes = Default::default();
    self.execution_order = Default::default();
    self.edges = Default::default();
    self.images.clear();
    self.views.clear();

    self.json_file = path.to_string();
    self.json_configuration = std::fs::read_to_string(path).expect("Unable to load json file!");
    let (nodes,
        execution,
        edges,
        dimensions,
        ip_port) = node_parser::parse_json(&self.interface, &self.json_configuration.as_str());
    
    self.network.bind(&ip_port.0, &ip_port.1);
    let meta = std::fs::metadata(&self.json_file);
    if let Ok(time) = meta.unwrap().modified() {
      self.json_timestamp = Some(time);
    }

    let img_info = gpu::ImageCreateInfo::builder()
    .gpu(0)
    .size(dimensions.0 as usize, dimensions.1 as usize)
    .format(gpu::ImageFormat::RGBA32F)
    .build();

    // Now, we need to fill out the node map with the names;
    let mut id = 0;
    for n in &nodes {
      self.node_name_map.insert(n.name(), id as usize);
      id += 1;
    }
    
    // Then, populate images.
    self.images.resize_with(nodes.len(), || {gpu::Image::new(&self.interface, &img_info)});
    self.views.reserve(self.images.len());

    for image in &self.images {
      self.views.push(image.view());
    }

    self.nodes = nodes;
    self.execution_order = execution;
    self.edges = edges;
    
    self.cmds[0].begin();
    for node_id in &self.execution_order {
      self.nodes[*node_id as usize].assign(&self.views[*node_id as usize]);
      // Send image if we have nodes who depend on it. Finishers should never get here.
      if self.edges.contains_key(node_id) {
        let entry = self.edges.get(node_id).unwrap();
        for id in entry {
          let view = &self.views[*id as usize];
          self.nodes[*node_id as usize].input(view);
        }
      }
      // Execute
      let node = & mut self.nodes[*node_id as usize];
      node.execute(& mut self.cmds[0]);
    }
    self.cmds[0].end();
  }

  pub fn new() -> Self {
    let num_layers = 1;

    let mut pipeline = Pipeline { 
      interface: gpu::GPUInterface::new(),
      images: Default::default(),
      views: Default::default(),
      nodes: Default::default(), 
      execution_order: Default::default(),
      edges: Default::default(),
      cmds: Default::default(),
      first: true,
      should_run: Rc::new(RefCell::new(true)),
      register: Default::default(),
      network: network::Manager::new(),
      node_name_map: Default::default(),
      json_file: Default::default(),
      json_configuration: Default::default(),
      json_timestamp: Default::default(),
    };

    let info = gpu::CommandListCreateInfo::builder()
    .gpu(0)
    .queue_type(gpu::QueueType::Graphics)
    .build();

    let mut cmds: Vec<gpu::CommandList> = Vec::new();
    for _i in 0..num_layers {
      cmds.push(gpu::CommandList::new(&pipeline.interface, &info));
    }

    pipeline.register = gpu::EventSubscriber::new(&pipeline.interface);

    pipeline.cmds = cmds;
    return pipeline;
  }

  pub fn should_run(&self) -> bool {
    return *self.should_run.as_ref().borrow();
  }

  fn handle_network(& mut self) {
    let result = self.network.receive_message();
    let mut req: Option<message::Request> = None;
    if result.is_some() {
      println!("Received message!");
      let msg = result.unwrap();
      let bytes = msg.bytes();
      let wrapped: std::result::Result<Vec<u8>, std::io::Error> = bytes.collect();
      if wrapped.is_ok() {
        println!("Received message");
        let raw = wrapped.unwrap();
        req = Some(message::Request::parse_from_bytes(&raw).expect("Received unknown message!"));
      }
    }

    if req.is_some() {
      let r = req.as_ref().unwrap();
      println!("Received message!");
      if r.request_type == message::RequestType::Image.into() {
        if !self.first {self.cmds[0].synchronize();}
        let name = r.node_name.clone();
        let node_id = self.node_name_map.get(&name);

        match node_id {
            Some(id) => {
              let mut response = message::Response::new();
              let mut img_response = response::ImageResponse::new();
              let img = &self.views[*id];
              let data = img.sync_get_pixels();
              response.description = "Image data from node".to_string();

              img_response.width = img.width() as i32;
              img_response.height = img.height() as i32;
              img_response.num_channels = 4;
              match data {
                gpu::ImagePixels::ImageF32(img) => {
                  img_response.image = img;
                },
                gpu::ImagePixels::ImageU8(_) => todo!(),
              }

              response.set_image_response(img_response);
              self.network.send_response(response);
            },
            None => todo!(),
        }
      } else if r.request_type == message::RequestType::Configuration.into() {
        let mut response = message::Response::new();
        let mut config_response = response::ConfigurationResponse::new();
        config_response.json = self.json_configuration.clone();
        response.set_config_response(config_response);
        self.network.send_response(response);
      } else {
        let name = r.node_name.clone();
        let node_id = self.node_name_map.get(&name);
        let response = self.nodes[*node_id.unwrap()].receive_message(& mut self.cmds[0], &r);
        self.network.send_response(response);
      }
    }
  }

  pub fn execute(& mut self) {
    if self.first {
      self.first = false;
      let s = self.should_run.clone();
      let cb = Box::new(move |event: &sdl2::event::Event| { 
        if event.is_window() {
          match event {
            sdl2::event::Event::Window { timestamp, window_id, win_event } => {
              if *win_event == sdl2::event::WindowEvent::Close {
                *s.borrow_mut() = false;
              }
            },
            sdl2::event::Event::Quit { .. } => *s.borrow_mut() = false,
            _ => {},
          }
        }
      });
      
      self.register.add_callback("RIP Quit Callback", cb);
    } else {
      self.cmds[0].synchronize();
    }
    
    self.cmds[0].submit();

    for node_id in &self.execution_order {
      self.nodes[*node_id as usize].post_execute(& mut self.cmds[0]);
    }

    self.handle_network();
    self.interface.as_ref().borrow_mut().poll_events();

    let meta = std::fs::metadata(&self.json_file);
    if meta.is_ok() {
      if let Ok(time) = meta.unwrap().modified() {
        if *self.json_timestamp.as_ref().unwrap() != time {
          let file = self.json_file.clone();
          self.parse_json(file.as_str());
        }
      }
    }
  }
}
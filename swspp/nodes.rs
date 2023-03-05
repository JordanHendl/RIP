extern crate runa;
extern crate json;
use std::{collections::{HashMap, HashSet, hash_map::Entry}, hash::Hash, cell::RefCell, rc::Rc};
use runa::{*, gpu::ImageView};
use json::*;
use crate::common::data_bus::DataBus;

use super::common;
use super::network;

mod finishers;
mod starters;
mod processors;
mod node_parser;

pub trait SwsppNode {
  fn name(&self) -> String;
  fn node_type(&self) -> String;
  fn assign(& mut self, view: &gpu::ImageView);
  fn input(& mut self, image: &gpu::ImageView); 
  fn execute(& mut self, cmd: & mut gpu::CommandList);
  fn post_execute(& mut self, _cmd: & mut gpu::CommandList) {}
}

// Information needed for any node to create itself.
pub struct NodeCreateInfo {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  name: String,
}

pub struct Pipeline {
  register: gpu::EventSubscriber,
  images: Vec<gpu::Image>,
  views: Vec<gpu::ImageView>,
  interface: Rc<RefCell<gpu::GPUInterface>>,
  nodes: Vec<Box<dyn SwsppNode + Send>>,
  execution_order: Vec<u32>,
  edges: HashMap<u32, Vec<u32>>,
  cmds: Vec<gpu::CommandList>,
  first: bool,
  should_run: Rc<RefCell<bool>>,
}

impl Pipeline {

  pub fn parse_json(& mut self, path: &str) {
    let (nodes,
        execution,
        edges,
        dimensions) = node_parser::parse_json(&self.interface, path);
    
    let img_info = gpu::ImageCreateInfo::builder()
    .gpu(0)
    .size(dimensions.0 as usize, dimensions.1 as usize)
    .format(gpu::ImageFormat::RGBA32F)
    .build();

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
      
      self.register.add_callback("SWSPP Quit Callback", cb);
    } else {
      self.cmds[0].synchronize();
    }
    
    self.cmds[0].submit();

    for node_id in &self.execution_order {
      self.nodes[*node_id as usize].post_execute(& mut self.cmds[0]);
    }

    self.interface.as_ref().borrow_mut().poll_events();
  }
}
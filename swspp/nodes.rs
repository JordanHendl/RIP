extern crate runa;
extern crate json;
use std::{collections::{HashMap, HashSet, hash_map::Entry}, hash::Hash, cell::RefCell, rc::Rc};
use runa::{*, gpu::ImageView};
use json::*;
use crate::common::data_bus::DataBus;

use super::common;
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
}

// Information needed for any node to create itself.
pub struct NodeCreateInfo {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  name: String,
}

pub struct Pipeline {
  images: Vec<gpu::Image>,
  views: Vec<gpu::ImageView>,
  interface: Rc<RefCell<gpu::GPUInterface>>,
  nodes: Vec<Box<dyn SwsppNode + Send>>,
  execution_order: Vec<u32>,
  edges: HashMap<u32, Vec<u32>>,
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


  }

  pub fn new() -> Self {
    let pipeline = Pipeline { 
      interface: gpu::GPUInterface::new(),
      images: Default::default(),
      views: Default::default(),
      nodes: Default::default(), 
      execution_order: Default::default(),
      edges: Default::default(),
    };

    return pipeline;
  }

  pub fn execute(& mut self) {
    let mut frame = self.interface.borrow_mut().begin_frame(0);
    let mut cmd = frame.command_list();
    cmd.begin();
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
      node.execute(& mut cmd);
    }

    cmd.end();
    frame.end_and_wait();
  }
}
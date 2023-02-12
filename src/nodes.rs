extern crate runa;
use std::collections::HashMap;
use runa::*;
use super::common;

mod agl;
use agl::*;


pub trait SwsppNode {
  fn name(&self) -> String;
  fn node_type(&self) -> String;
  fn execute(&self, cmd: &gpu::CommandList);
}

pub struct NodeCreateInfo {
  interface: * mut gpu::GPUInterface,
}

type Callback = fn(&NodeCreateInfo) -> Box<dyn SwsppNode>;

pub struct Pipeline {
  node_creation_functions: HashMap<String, Callback>,
  nodes: Vec<Box<dyn SwsppNode + Send>>,
  execution_order: Vec<u32>,
}

impl Pipeline {
  pub fn new(config_path: &str) -> Self {
    let mut functors: HashMap<String, Callback> = Default::default();
    functors.insert("agl".to_string(), agl::AGL::new);

      return Pipeline { 
        node_creation_functions: functors,
        nodes: Default::default(), 
        execution_order: Default::default() 
      }
  }
}
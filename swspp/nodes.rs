extern crate runa;
extern crate json;
use std::{collections::{HashMap, HashSet, hash_map::Entry}, hash::Hash, cell::RefCell, rc::Rc};
use runa::*;
use json::*;
use crate::common::data_bus::DataBus;

use super::common;
mod finishers;
mod starters;
mod agl;
use agl::*;


pub trait SwsppNode {
  fn name(&self) -> String;
  fn node_type(&self) -> String;
  fn input(& mut self, image: &gpu::ImageView);
  fn output(&self) -> gpu::ImageView;
  fn execute(& mut self, cmd: &gpu::CommandList);
}

// Information needed for any node to create itself.
pub struct NodeCreateInfo {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  name: String,
}

type Callback = fn(&NodeCreateInfo) -> Box<dyn SwsppNode + Send>;

pub struct Pipeline {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  nodes: Vec<Box<dyn SwsppNode + Send>>,
  execution_order: Vec<u32>,
  edges: HashMap<u32, Vec<u32>>,
}

impl Pipeline {
  fn get_starter_functors() -> HashMap<String, Callback> {
    let mut functors: HashMap<String, Callback> = Default::default();
    functors.insert("image_load".to_string(), starters::ImageLoad::new);
    return functors;
  }

  fn get_finisher_functors() -> HashMap<String, Callback> {
    let mut functors: HashMap<String, Callback> = Default::default();
    functors.insert("image_write".to_string(), finishers::ImageWrite::new);
    return functors;
  } 
  fn get_functors() -> HashMap<String, Callback> {
    let mut functors: HashMap<String, Callback> = Default::default();
    functors.insert("agl".to_string(), agl::AGL::new);
    return functors;
  }

  fn configure_nodes(json: &JsonValue) {
    let data_bus: DataBus = Default::default();
    for node in json.entries() {
      for config in node.1.entries() {
        if config.0.ne("type") {
          let key = node.0.to_string() + &"::".to_string() + config.0;
          if config.1.is_boolean() {
            data_bus.send(&key, &config.1.as_bool().unwrap());
          } else if config.1.is_string() {
            println!("Sending configuration {} for {}", config.1.as_str().unwrap(), &key);
            data_bus.send(&key, &config.1.as_str().unwrap().to_string());
          }
        }
      }
    }
  }

  pub fn parse_json(& mut self, path: &str) {
    let starter_functors = Pipeline::get_starter_functors();
    let node_functors = Pipeline::get_functors();
    let finisher_functors = Pipeline::get_finisher_functors();

    let json_data = std::fs::read_to_string(path).expect("Unable to load json file!");
    let mut created_nodes:  Vec<Box<dyn SwsppNode + Send>> = Vec::new();

    let mut starter_ids: HashMap<String, usize> = HashMap::new();
    let mut node_ids: HashMap<String, usize> = HashMap::new();
    let mut finisher_ids: HashMap<String, usize> = HashMap::new();

    let mut connections: HashMap<u32, Vec<u32>> = HashMap::new();

    println!("Loaded json file: \n{}", json_data);
    let root = json::parse(json_data.as_str()).expect("Unable to parse JSON configuration!");
    assert!(root.has_key("starters"), "Failed to find any pipeline starters in the configuration!");
    assert!(root.has_key("finishers"), "Failed to find any pipeline finishers in the configuration!");
    //assert!(root.has_key("nodes"), "Failed to find any pipeline nodes in the configuration!");

    let starters = &root["starters"];
    let nodes = &root["nodes"];
    let finishers = &root["finishers"];
    
    let mut node_handler = |node: (&str, &JsonValue), functors: &HashMap<String, Callback>| {
      let name = node.0;
      let type_name = if node.1.has_key("type") {node.1["type"].as_str()} else {Some(node.0)}.unwrap();
      println!("JSON: Parsing {} of type {}...", name, type_name);

      if functors.contains_key(type_name) {
        let create_info = NodeCreateInfo {
          interface: self.interface.clone(),
          name: name.to_string(),
        };
        let node = functors[type_name](&create_info);
        return Some(node);
      }
      return None;
    };

    for node in starters.entries() {
      let created_node = node_handler(node, &starter_functors);
      if created_node.is_some() {
        let n = created_node.unwrap();
        let name = n.name().clone();
        created_nodes.push(n);
        starter_ids.insert(name, created_nodes.len() - 1);
      }
    }
    
    
    for node in nodes.entries() {
      let created_node = node_handler(node, &node_functors);
      if created_node.is_some() {
        let n = created_node.unwrap();
        let name = n.name().clone();
        created_nodes.push(n);
        node_ids.insert(name, created_nodes.len() - 1);
      }
    }

    for node in finishers.entries() {
      let created_node = node_handler(node, &finisher_functors);
      if created_node.is_some() {
        let n = created_node.unwrap();
        let name = n.name().clone();
        created_nodes.push(n);
        finisher_ids.insert(name, created_nodes.len() - 1);
      }
    }
    
    Pipeline::configure_nodes(starters);
    Pipeline::configure_nodes(finishers);

    // Now to find execution order
    let mut execution_order: Vec<u32> = Vec::new();
    let mut inserted_nodes: HashSet<String> = HashSet::new();

    // We can safetly execute all starters as they have no preconditions
    for starter in &starter_ids {
      execution_order.push(*starter.1 as u32);
      inserted_nodes.insert(starter.0.clone());
    }

    let mut num_inserted_nodes = 0;
    let mut insert_fn = |node: (&str, &JsonValue) , ids: &HashMap<String, usize>| {
      assert!(node.1.has_key("input"));
      let entry = ids.get(node.0).unwrap();
      if !inserted_nodes.contains(&node.0.to_string()) {
        let input = node.1["input"].as_str().unwrap();
        let is_an_output = input.find(".output");
        if is_an_output.is_none() {
          panic!("Some node doesn't have it's input specified correctly. Inputs should come in the form 'input: \"other_node.output\"'");
        }
        
        // Find the node name from the input
        let input_name: String = input.chars().take(is_an_output.unwrap()).collect();
        
        // This node depends on a starter, so we can safetly just have them execute whenever.
        if inserted_nodes.contains(&input_name) {
          execution_order.push(*entry as u32);
          inserted_nodes.insert(node.0.to_string());
          
          // Bro wtf is this
          let parent_id = if starter_ids.contains_key(&input_name.to_string()) 
          {*starter_ids.get(&input_name.to_string()).unwrap() as u32} else 
          {*ids.get(&input_name.to_string()).unwrap() as u32};

          // Map connections so we know where to send data as we execute nodes.          
          match connections.entry(parent_id) {
            Entry::Occupied(mut o) => {
              println!("Adding connection between node {} -> {}", parent_id, execution_order.len() - 1);
              o.get_mut().push(execution_order.len() as u32 - 1);
            },
            Entry::Vacant(v) => {
              println!("Adding connection between node {} -> {}", parent_id, execution_order.len() - 1);
              let mut vec: Vec<u32> = Vec::new();
              vec.push(execution_order.len() as u32 - 1);
              v.insert(vec);
            },
          }
          return true;
        }
      }

      return false;
    };

    
    // Loop until we get all the nodes inserted.
    while num_inserted_nodes != node_ids.len() {
      for node in nodes.entries() {
        if insert_fn(node, &node_ids) {
          num_inserted_nodes += 1;
        }
      }
    }

    num_inserted_nodes = 0;
    while num_inserted_nodes != finisher_ids.len() {
      for node in finishers.entries() {
        if insert_fn(node, &finisher_ids) {
          num_inserted_nodes += 1;
        }
      }
    }


    self.nodes = created_nodes;
    self.execution_order = execution_order;
    self.edges = connections;
  }

  pub fn new() -> Self {
    let mut pipeline = Pipeline { 
      interface: gpu::GPUInterface::new(),
      nodes: Default::default(), 
      execution_order: Default::default(),
      edges: Default::default(),
    };

    return pipeline;
  }

  pub fn execute(& mut self) {
    let mut frame = self.interface.borrow_mut().begin_frame(0);
    let mut cmd = frame.command_list();
    for node_id in &self.execution_order {
      let node = & mut self.nodes[*node_id as usize];

      // Execute
      node.execute(&cmd);

      // Send image if we have nodes who depend on it. Finishers should never get here.
      if self.edges.contains_key(node_id) {
        let img = node.output();
        let entry = self.edges.get(node_id).unwrap();
        for id in entry {
          self.nodes[*id as usize].input(&img);
        }
      }
    }
    frame.end();
  }
}
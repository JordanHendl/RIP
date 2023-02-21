extern crate runa;
extern crate json;
use std::{collections::{HashMap, HashSet, hash_map::Entry}, cell::RefCell, rc::Rc};
use runa::*;
use json::*;
use crate::common::data_bus::DataBus;

use super::{NodeCreateInfo, SwsppNode, starters, finishers, processors};

type Callback = fn(&NodeCreateInfo) -> Box<dyn SwsppNode + Send>;
fn get_reserved_strings() -> HashMap<String, String> {
  let src_dir = concat!(env!("CARGO_MANIFEST_DIR"));
  let mut reserved: HashMap<String, String> = Default::default();

  reserved.insert("${SRC_DIR}".to_string(), src_dir.to_string());
  return reserved;
}

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
  functors.insert("agl".to_string(), processors::AGL::new);
  functors.insert("intensify".to_string(), processors::Intensify::new);
  functors.insert("inverse".to_string(), processors::Inverse::new);
  functors.insert("blur".to_string(), processors::Blur::new);
  return functors;
}

fn find_connections(starter_ids: &HashMap<String, usize>, node_ids: &HashMap<String, usize>, finisher_ids: &HashMap<String, usize>,
  starters: &JsonValue, nodes: &JsonValue, finishers: &JsonValue) -> (HashMap<u32, Vec<u32>>, Vec<u32>) {
  // Now to find execution order
  let mut execution_order: Vec<u32> = Vec::new();
  let mut inserted_nodes: HashSet<String> = HashSet::new();
  let mut connections: HashMap<u32, Vec<u32>> = HashMap::new();
  
  // We can safetly execute all starters as they have no preconditions
  for starter in starter_ids {
    execution_order.push(*starter.1 as u32);
    inserted_nodes.insert(starter.0.clone());
  }
  let mut num_inserted_nodes = 0;
  
  let mut insert_fn = |node: (&str, &JsonValue) , ids: &HashMap<String, usize>| {
    assert!(node.1.has_key("input"));
    
    let mut num_deps = 1;
    let entry = ids.get(node.0).unwrap();
    if !inserted_nodes.contains(&node.0.to_string()) {
      let mut has_all_deps = 0;
      let mut dep_names:Vec<String> = Vec::new();
      
      
      if node.1.is_array() {
        num_deps = node.1.len();
        let node_inputs = &node.1["input"];
        for i in 0..node_inputs.len() {
          let input_name = node_inputs[i].as_str().unwrap();
          if inserted_nodes.contains(input_name) {
            has_all_deps += 1;
            dep_names.push(input_name.to_string());
          }
        }
      } else {
        let input = node.1["input"].as_str().as_ref().unwrap().to_string().clone();
        let is_an_output = input.find(".output");
        if is_an_output.is_none() {
          panic!("Some node doesn't have it's input specified correctly. Inputs should come in the form 'input: \"other_node.output\"'");
        }
      
        // Find the node name from the input
        let input_name: String = input.chars().take(is_an_output.unwrap()).collect();
        
        // This node depends on a starter, so we can safetly just have them execute whenever.
        if inserted_nodes.contains(&input_name) {
          has_all_deps += 1;
          dep_names.push(input_name.clone());
        }
      }
      
      if has_all_deps == num_deps {
        execution_order.push(*entry as u32);
        inserted_nodes.insert(node.0.to_string());
        
        for parent_name in dep_names {
          // Bro wtf is this
          let parent_id = if starter_ids.contains_key(&parent_name.to_string()) 
          {*starter_ids.get(&parent_name.to_string()).unwrap() as u32} else 
          {*node_ids.get(&parent_name.to_string()).unwrap() as u32};
          
          let self_id = (execution_order.len() - 1) as u32;
          // Map connections so we know where to send data as we execute nodes.          
          match connections.entry(self_id) {
            Entry::Occupied(mut o) => {
            println!("Adding connection between node {} -> {}", parent_id, self_id);
            o.get_mut().push(parent_id);
          },
          Entry::Vacant(v) => {
            println!("Adding connection between node {} -> {}", parent_id, execution_order.len() - 1);
            let mut vec: Vec<u32> = Vec::new();
            vec.push(parent_id);
            v.insert(vec);
          },
          }
        }
        println!("Found all deps for {}!", node.0);
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
  
  return (connections, execution_order);
}

fn configure_nodes(json: &JsonValue) {
  let data_bus: DataBus = Default::default();
  let reserved_strings = get_reserved_strings();
  for node in json.entries() {
    for config in node.1.entries() {
      if config.0.ne("type") {
        let key = node.0.to_string() + &"::".to_string() + config.0;
        if config.1.is_boolean() {
          data_bus.send(&key, &config.1.as_bool().unwrap());
        } else if config.1.is_string() {

          let mut value = config.1.as_str().unwrap().to_string().clone();
          for string in &reserved_strings {
            value = value.replace(string.0, string.1);
          }
          
          println!("Sending configuration {} for {}", value, &key);
          data_bus.send(&key, &value);
        }
      }
    }
  }
}

pub fn parse_json(interface: &Rc<RefCell<gpu::GPUInterface>>, path: &str) -> 
(Vec<Box<dyn SwsppNode + Send>>, Vec<u32>, HashMap<u32, Vec<u32>>) {

  let starter_functors = get_starter_functors();
  let node_functors = get_functors();
  let finisher_functors = get_finisher_functors();

  let json_data = std::fs::read_to_string(path).expect("Unable to load json file!");
  let mut created_nodes:  Vec<Box<dyn SwsppNode + Send>> = Vec::new();

  let mut starter_ids: HashMap<String, usize> = HashMap::new();
  let mut node_ids: HashMap<String, usize> = HashMap::new();
  let mut finisher_ids: HashMap<String, usize> = HashMap::new();

  println!("Loaded json file: \n{}", json_data);
  let root = json::parse(json_data.as_str()).expect("Unable to parse JSON configuration!");
  assert!(root.has_key("starters"), "Failed to find any pipeline starters in the configuration!");
  assert!(root.has_key("finishers"), "Failed to find any pipeline finishers in the configuration!");

  let starters = &root["starters"];
  let nodes = &root["imgproc"];
  let finishers = &root["finishers"];
  
  let node_handler = |node: (&str, &JsonValue), functors: &HashMap<String, Callback>| {
    let name = node.0;
    let type_name = if node.1.has_key("type") {node.1["type"].as_str()} else {Some(node.0)}.unwrap();
    println!("JSON: Parsing {} of type {}...", name, type_name);

    if functors.contains_key(type_name) {
      let create_info = NodeCreateInfo {
        interface: interface.clone(),
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

      
  configure_nodes(starters);
  configure_nodes(nodes);
  configure_nodes(finishers);

  let (connections, execution_order) = find_connections(
    &starter_ids, 
    &node_ids, 
    &finisher_ids, 
    &starters, 
    &nodes, 
    &finishers);


  return (created_nodes, execution_order, connections);
}
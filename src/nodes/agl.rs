extern crate runa;
use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;

#[derive(Default)]
struct AGLData {
  pipeline: gpu::ComputePipeline,
}

pub struct AGL {
  interface: * mut gpu::GPUInterface,
  data: AGLData,
  data_bus: crate::common::DataBus,
}

impl SwsppNode for AGL {
  fn execute(&self, cmd: &gpu::CommandList) {
      
  }

  fn name(&self) -> String {
    return "SomeType".to_string();
  }

  fn node_type(&self) -> String {
    return "AGL".to_string();
  }
}

impl AGL {
  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode> {
    let agl = AGL {
      interface: info.interface,
      data: Default::default(),
      data_bus: Default::default(),
    };

    return Box::new(agl);
  }
}
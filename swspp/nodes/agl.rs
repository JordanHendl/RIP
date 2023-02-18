extern crate runa;
use std::cell::RefCell;
use std::rc::Rc;
use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;

#[derive(Default)]
struct AGLData {
  pipeline: gpu::ComputePipeline,
}

pub struct AGL {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: AGLData,
  data_bus: crate::common::DataBus,
}

impl SwsppNode for AGL {
  fn execute(& mut self, cmd: &gpu::CommandList) {
      
  }
  
  fn input(& mut self, image: &gpu::ImageView) {

  }

  fn output(&self) -> gpu::ImageView {
    return Default::default();
  }

  fn name(&self) -> String {
    return "SomeType".to_string();
  }

  fn node_type(&self) -> String {
    return "AGL".to_string();
  }
}
unsafe impl Send for AGL {}
impl AGL {
  fn set_min_rad(& mut self) {
    //todo
  }
  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
//    let data_bus: DataBus = Default::default();
//    data_bus.add_object_subscriber(&info.name + "::min_rad", &self, AGL::set_min_rad);

    let agl = AGL {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
    };

    println!("Calling AGL callback!");
    return Box::new(agl);
  }
}
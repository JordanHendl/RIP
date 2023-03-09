extern crate runa;
use std::cell::RefCell;
use std::rc::Rc;
use crate::common;

use super::NodeCreateInfo;
use super::RipNode;
use super::common::*;
use runa::*;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////

#[derive(Default)]
struct ArithmeticConfig {
  mode: u32,
}

#[derive(Default)]
struct ArithmeticData {
  num_received_inputs: u32,
  image: Option<gpu::ImageView>,
  config: gpu::Vector<ArithmeticConfig>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct Arithmetic {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ArithmeticData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Arithmetic {}

// Implementations specific to this node
impl Arithmetic {
    fn set_mode(& mut self, input: &String) {
    println!("Setting mode {} for node {}", input, self.name);
    let mut mode = 0;
    match input.as_str() {
      "subtract" => mode = 0,
      "add" => mode = 1,
      "multiply" => mode = 2,
      "divide" => mode = 3,
      _ => {},
    }

    let mapped = unsafe{self.data.config.map()};
    mapped[0].mode = mode;
    unsafe{self.data.config.unmap()};
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    println!("Creating node {} as an Arithmetic node!", info.name);
    let mut obj = Box::new(Arithmetic {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();
  
    let default_config: ArithmeticConfig = Default::default();
    obj.data.config = gpu::Vector::new(&obj.interface, &buff_info);
    obj.data.config.upload(std::slice::from_ref(&default_config));

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/arithmetic.spirv")).as_ref());
    let info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader)
    .name(&info.name)
    .build();

    let pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.bind_group = pipeline.bind_group();
    obj.data.pipeline = pipeline;
    obj.data.bind_group.bind_vector("config", &obj.data.config);

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name + "::mode"), obj.as_mut(), Arithmetic::set_mode);
    obj.data_bus = bus;

    return obj;
  }
}

// Base class implementations
impl RipNode for Arithmetic {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    assert!(self.data.num_received_inputs == 2, "Trying to use an Arithmetic node without giving it the required amount of inputs (2).");
    println!("Executing Node {}", self.name);
    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(x, y, z);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
    self.data.num_received_inputs = 0;
  }

  fn input(& mut self, image: &gpu::ImageView) {
    if self.data.num_received_inputs == 0 {
      self.data.bind_group.bind_image_view("input_tex_0", image);
      self.data.num_received_inputs += 1;
    } else if self.data.num_received_inputs == 1 {
      self.data.bind_group.bind_image_view("input_tex_1", image);
      self.data.num_received_inputs += 1;
    }
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    self.data.image = Some(view.clone());
    self.data.bind_group.bind_image_view("output_tex", &self.data.image.as_ref().unwrap());
  }


  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "arithmetic".to_string();
  }
}
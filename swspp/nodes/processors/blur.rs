extern crate runa;
use std::cell::RefCell;
use std::rc::Rc;
use crate::common;

use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////

struct BlurConfig {
  radius: u32,
}

impl Default for BlurConfig {
  fn default() -> Self {
      return BlurConfig { radius: 5 }
  }
}

#[derive(Default)]
struct BlurData {
  image: Option<gpu::ImageView>,
  config: Option<gpu::Vector<BlurConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct Blur {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: BlurData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Blur {}

// Implementations specific to this node
impl Blur {
  pub fn set_radius(& mut self, radius: &u32) {
    println!("Setting radius {} for node {}", *radius, self.name);
    let default_config: BlurConfig = BlurConfig { radius: *radius };
    self.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    let mut obj = Box::new(Blur {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/blur.spirv")).as_ref());
    let info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader)
    .name(&info.name)
    .build();

    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();
    
    let default_config: BlurConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.bind_group = pipeline.bind_group();
    obj.data.pipeline = pipeline;
    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name + "::radius"), obj.as_mut(), Blur::set_radius);
    obj.data_bus = bus;

    return obj;
  }
}

// Base class implementations
impl SwsppNode for Blur {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(x, y, z);
  }

  fn input(& mut self, image: &gpu::ImageView) {
    self.data.bind_group.bind_image_view("input_tex", image);
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    self.data.image = Some(view.clone());
    self.data.bind_group.bind_image_view("output_tex", &self.data.image.as_ref().unwrap());
  }

  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "blur".to_string();
  }
}
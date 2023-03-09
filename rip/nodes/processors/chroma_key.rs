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

#[repr(C)]
struct ChromaKeyConfig {
  r: f32,
  g: f32,
  b: f32,
  low_range: f32,
  high_range: f32,
}

#[derive(Default)]
struct ChromaKeyData {
  num_received_inputs: u32,
  image: Option<gpu::ImageView>,
  config: Option<gpu::Vector<ChromaKeyConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct ChromaKey {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ChromaKeyData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for ChromaKey {}

impl Default for ChromaKeyConfig {
  fn default() -> Self {
      return ChromaKeyConfig { r: 0.0, g: 1.0, b: 0.0, low_range: 0.1, high_range: 0.5}
  }
}

// Implementations specific to this node
impl ChromaKey {
  pub fn set_r(& mut self, v: &u32) {
    println!("Setting red value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].r = *v as f32 / 256.0;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_g(& mut self, v: &u32) {
    println!("Setting green value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].g = *v as f32 / 256.0;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_b(& mut self, v: &u32) {
    println!("Setting blue value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].b = *v as f32 / 256.0;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_low_range(& mut self, v: &f32) {
    println!("Setting low range {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].low_range = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_high_range(& mut self, v: &f32) {
    println!("Setting high range {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].high_range = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(ChromaKey {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/chroma_key.spirv")).as_ref());
    let info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader)
    .name(&info.name)
    .build();

    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();

    let pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.bind_group = pipeline.bind_group();
    obj.data.pipeline = pipeline;

    let default_config: ChromaKeyConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name.clone() + "::red"), obj.as_mut(), ChromaKey::set_r);
    bus.add_object_subscriber(&(name.clone() + "::green"), obj.as_mut(), ChromaKey::set_g);
    bus.add_object_subscriber(&(name.clone() + "::blue"), obj.as_mut(), ChromaKey::set_b);
    bus.add_object_subscriber(&(name.clone() + "::low_range"), obj.as_mut(), ChromaKey::set_low_range);
    bus.add_object_subscriber(&(name.clone() + "::high_range"), obj.as_mut(), ChromaKey::set_high_range);
    obj.data_bus = bus;

    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());
    return obj;
  }
}

// Base class implementations
impl RipNode for ChromaKey {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    assert!(self.data.num_received_inputs == 2, "Trying to use a chroma key node without giving it the required amount of inputs (2).");
    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(x, y, z);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
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
    return "chroma_key".to_string();
  }
}
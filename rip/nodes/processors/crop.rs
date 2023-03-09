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
struct CropConfig {
  top: u32,
  left: u32,
  bottom: u32,
  right: u32,
}

#[derive(Default)]
struct CropData {
  image: Option<gpu::ImageView>,
  config: Option<gpu::Vector<CropConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct Crop {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: CropData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Crop {}

impl Default for CropConfig {
  fn default() -> Self {
      return CropConfig { left: 20, right: 20, top: 20, bottom: 20,}
  }
}

// Implementations specific to this node
impl Crop {
  pub fn set_top(& mut self, v: &u32) {
    println!("Setting top value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].top = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_left(& mut self, v: &u32) {
    println!("Setting left value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].left = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_bottom(& mut self, v: &u32) {
    println!("Setting bottom value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].bottom = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn set_right(& mut self, v: &u32) {
    println!("Setting right value {} for node {}", v, self.name);
    let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
    mapped[0].right = *v;
    unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(Crop {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/crop.spirv")).as_ref());
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

    let default_config: CropConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name.clone() + "::top"), obj.as_mut(), Crop::set_top);
    bus.add_object_subscriber(&(name.clone() + "::left"), obj.as_mut(), Crop::set_left);
    bus.add_object_subscriber(&(name.clone() + "::bottom"), obj.as_mut(), Crop::set_bottom);
    bus.add_object_subscriber(&(name.clone() + "::right"), obj.as_mut(), Crop::set_right);
    obj.data_bus = bus;

    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());
    return obj;
  }
}

// Base class implementations
impl RipNode for Crop {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(x, y, z);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
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
    return "crop".to_string();
  }
}
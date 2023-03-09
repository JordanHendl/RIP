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
struct OverlayConfig {
  dummy: u32,
}

#[derive(Default)]
struct OverlayData {
  num_received_inputs: u32,
  image: Option<gpu::ImageView>,
  config: Option<gpu::Vector<OverlayConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct Overlay {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: OverlayData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Overlay {}

impl Default for OverlayConfig {
  fn default() -> Self {
      return OverlayConfig { dummy: 0,}
  }
}

// Implementations specific to this node
impl Overlay {
  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(Overlay {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/overlay.spirv")).as_ref());
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

    let default_config: OverlayConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    obj.data_bus = bus;

    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());
    return obj;
  }
}

// Base class implementations
impl RipNode for Overlay {
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
    return "overlay".to_string();
  }
}
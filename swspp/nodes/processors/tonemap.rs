extern crate runa;
use std::cell::RefCell;
use std::rc::Rc;
use crate::common;
use crate::gpgpu;

use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////

struct TonemapConfig {
  mode: u32,
}

#[derive(Default)]
struct TonemapData {
  input: Option<gpu::ImageView>,
  image: Option<gpu::ImageView>,
  config: gpu::Vector<TonemapConfig>,
  histogram: Option<gpgpu::Histogram>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct Tonemap {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: TonemapData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Tonemap {}

impl Default for TonemapConfig {
  fn default() -> Self {
    TonemapConfig { mode: 0 }
  }
}

// Implementations specific to this node
impl Tonemap {
  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    let mut obj = Box::new(Tonemap {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/tonemap.spirv")).as_ref());
    let info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader)
    .name(&info.name)
    .build();

    let hist = gpgpu::Histogram::new(&obj.interface, Default::default());
    
    let pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.bind_group = pipeline.bind_group();
    obj.data.pipeline = pipeline;
    
    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();

    let default_config: TonemapConfig = Default::default();
    let mut config: gpu::Vector<TonemapConfig> = gpu::Vector::new(&obj.interface, &buff_info);
    config.upload(std::slice::from_ref(&default_config));

    obj.data.bind_group.bind_vector("hist_config", hist.config());
    obj.data.bind_group.bind_vector("hist_data", hist.histogram());
    obj.data.bind_group.bind_vector("config", &config);

    obj.data.histogram = Some(hist);
    obj.data.config = config;
    return obj;
  }
}

// Base class implementations
impl SwsppNode for Tonemap {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);

    if self.data.input.is_some() {
      self.data.histogram.as_mut().unwrap().calculate(self.data.input.as_ref().unwrap(), cmd);
    }

    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(x, y, z);
  }

  fn input(& mut self, image: &gpu::ImageView) {
    self.data.input = Some(image.clone());
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
    return "tonemap".to_string();
  }
}
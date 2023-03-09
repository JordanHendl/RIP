extern crate runa;
use std::cell::RefCell;
use std::rc::Rc;
use crate::common;
use crate::gpgpu;

use super::NodeCreateInfo;
use super::RipNode;
use super::common::*;
use runa::*;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////
/// 
#[repr(C)]
struct TonemapConfig {
  mode: u32,
}

#[derive(Default)]
struct TonemapData {
  input: Option<gpu::ImageView>,
  image: Option<gpu::ImageView>,

  tonecurve: gpu::Vector<f32>,
  config: gpu::Vector<TonemapConfig>,

  histogram: Option<gpgpu::Histogram>,

  generate_tonecurve_pipeline: gpu::ComputePipeline,
  apply_tonecurve_pipeline: gpu::ComputePipeline,

  generate_tonecurve_bg: gpu::BindGroup,
  apply_tonecurve_bg: gpu::BindGroup,
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
  fn set_num_bins(& mut self, bins: &u32) {
    self.data.histogram.as_mut().unwrap().set_num_bins(*bins);

    if self.data.input.is_none() {
      let mut buff_info = gpu::BufferCreateInfo::builder()
      .gpu(0)
      .size(self.data.histogram.as_ref().unwrap().histogram().len())
      .build();
  
      let tonecurve: gpu::Vector<f32> = gpu::Vector::new(&self.interface, &buff_info);
      self.data.tonecurve = tonecurve;
    }
  }

  fn set_max_rad(& mut self, max_rad: &f32) {
    self.data.histogram.as_mut().unwrap().set_max_rad(*max_rad);
  }

  fn set_min_rad(& mut self, min_rad: &f32) {
    self.data.histogram.as_mut().unwrap().set_min_rad(*min_rad);
  }

  fn set_mode(& mut self, input: &String) {
    println!("Setting mode {} for node {}", input, self.name);
    let mut mode = 0;
    match input.as_str() {
      "normalized" => mode = 0,
      _ => {},
    }

    let mapped = unsafe{self.data.config.map()};
    mapped[0].mode = mode;
    unsafe{self.data.config.unmap()};
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(Tonemap {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader_0 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/tonemap_generate_curve.spirv")).as_ref());
    let raw_shader_1 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/tonemap.spirv")).as_ref());
    let mut info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader_0)
    .name(&info.name)
    .build();

    let hist = gpgpu::Histogram::new(&obj.interface, Default::default());
    
    let generate_tonecurve_pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.generate_tonecurve_bg = generate_tonecurve_pipeline.bind_group();

    info.shader = (&raw_shader_1).to_vec();

    let apply_tonecurve_pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.apply_tonecurve_bg = apply_tonecurve_pipeline.bind_group();
    
    let mut buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();

    let default_config: TonemapConfig = Default::default();
    let mut config: gpu::Vector<TonemapConfig> = gpu::Vector::new(&obj.interface, &buff_info);
    config.upload(std::slice::from_ref(&default_config));

    buff_info.size = hist.histogram().len();
    let tonecurve: gpu::Vector<f32> = gpu::Vector::new(&obj.interface, &buff_info);
    
    obj.data.apply_tonecurve_pipeline = apply_tonecurve_pipeline;
    obj.data.generate_tonecurve_pipeline = generate_tonecurve_pipeline;

    obj.data.generate_tonecurve_bg.bind_vector("hist_config", hist.config());
    obj.data.generate_tonecurve_bg.bind_vector("hist_data", hist.histogram());
    obj.data.generate_tonecurve_bg.bind_vector("data", &tonecurve);
    obj.data.generate_tonecurve_bg.bind_vector("config", &config);

    obj.data.apply_tonecurve_bg.bind_vector("hist_config", hist.config());
    obj.data.apply_tonecurve_bg.bind_vector("hist_data", hist.histogram());
    obj.data.apply_tonecurve_bg.bind_vector("data", &tonecurve);
    obj.data.apply_tonecurve_bg.bind_vector("config", &config);

    obj.data.histogram = Some(hist);
    obj.data.config = config;
    obj.data.tonecurve = tonecurve;

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name.clone() + "::mode"), obj.as_mut(), Tonemap::set_mode);
    bus.add_object_subscriber(&(name.clone() + "::num_bins"), obj.as_mut(), Tonemap::set_num_bins);
    bus.add_object_subscriber(&(name.clone() + "::max_rad"), obj.as_mut(), Tonemap::set_max_rad);
    bus.add_object_subscriber(&(name.clone() + "::min_rad"), obj.as_mut(), Tonemap::set_min_rad);
    obj.data_bus = bus;

    return obj;
  }
}

// Base class implementations
impl RipNode for Tonemap {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);

    if self.data.input.is_some() {
      self.data.histogram.as_mut().unwrap().calculate(self.data.input.as_ref().unwrap(), cmd);
    }

    let x = self.data.histogram.as_ref().unwrap().histogram().get_compute_groups(32);
    cmd.bind_compute(&self.data.generate_tonecurve_pipeline);
    cmd.bind(&self.data.generate_tonecurve_bg);
    cmd.dispatch(x, 1, 1);
    cmd.vector_write_barrier(&self.data.tonecurve);

    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.apply_tonecurve_pipeline);
    cmd.bind(&self.data.apply_tonecurve_bg);
    cmd.dispatch(x, y, z);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
  }

  fn input(& mut self, image: &gpu::ImageView) {
    self.data.input = Some(image.clone());
    self.data.apply_tonecurve_bg.bind_image_view("input_tex", image);
    self.data.generate_tonecurve_bg.bind_image_view("input_tex", image);
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    self.data.image = Some(view.clone());
    self.data.apply_tonecurve_bg.bind_image_view("output_tex", &self.data.image.as_ref().unwrap());
  }

  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "tonemap".to_string();
  }
}
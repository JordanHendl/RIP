extern crate runa;
use std::{cell::RefCell, rc::Rc};
use runa::*;

use crate::common::to_u32_slice;

#[derive(Copy, Clone)]
pub struct HistogramConfig {
  num_bins: u32,
  min_rad: f32,
  max_rad: f32,
}

impl Default for HistogramConfig {
  fn default() -> Self {
      return HistogramConfig { num_bins: 64, min_rad: 0.0, max_rad: 1.0 }
  }
}

#[derive(Default)]
struct HistogramData {
  calculated: bool,
  histogram: gpu::Vector<u32>,
  config: gpu::Vector<HistogramConfig>,
  clear_hist: gpu::ComputePipeline,
  calculate_hist: gpu::ComputePipeline,
  clear_bg: gpu::BindGroup,
  calculate_bg: gpu::BindGroup,
}

pub struct Histogram {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: HistogramData,
}

impl Histogram {
  pub fn new(interface: &Rc<RefCell<gpu::GPUInterface>>, cfg: HistogramConfig) -> Self {
    let clear_histogram = to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/clear_histogram.spirv")).as_ref());
    let histogram = to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/histogram.spirv")).as_ref());
    let mut pipe_info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .name("Clear Histogram")
    .shader(clear_histogram)
    .build();

    let clear_pipe = gpu::ComputePipeline::new(interface, &pipe_info);

    pipe_info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .name("Calc Histogram")
    .shader(histogram)
    .build();
    
    let calc_pipe = gpu::ComputePipeline::new(interface, &pipe_info);
    let mut buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();
  
    let config: HistogramConfig = cfg.clone();
    let mut cfg: gpu::Vector<HistogramConfig> = gpu::Vector::new(&interface, &buff_info);
    cfg.upload(std::slice::from_ref(&config));
    
    buff_info.size = config.num_bins as usize;

    let mut data: gpu::Vector<u32> = gpu::Vector::new(&interface, &buff_info);
    let default_data: Vec<u32> = vec![0 as u32; config.num_bins as usize];
    data.upload(&default_data);

    let bg1 = clear_pipe.bind_group();
    bg1.bind_vector("data", &data);

    let bg2 = calc_pipe.bind_group();
    bg2.bind_vector("config", &cfg);
    bg2.bind_vector("data", &data);

    let hist_data = HistogramData {
      histogram: data,
      clear_hist: clear_pipe,
      calculate_hist: calc_pipe,
      config: cfg,
      clear_bg: bg1,
      calculate_bg: bg2,
      calculated: false,
    };

    let hist = Histogram {
      interface: interface.clone(),
      data: hist_data,
    };
    return hist;
  }

  pub fn calculate(& mut self, img: &gpu::ImageView, cmd: &gpu::CommandList) {
    self.data.calculated = true;
    self.data.calculate_bg.bind_image_view("input_tex", &img);
    let x = self.data.histogram.get_compute_groups(32);
    cmd.bind_compute(&self.data.clear_hist);
    cmd.bind(&self.data.clear_bg);
    cmd.dispatch(x, 1, 1);

    let (x, y, z) = img.get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.calculate_hist);
    cmd.bind(&self.data.calculate_bg);
    cmd.dispatch(x, y, z);
  }

  pub fn config(&self) -> &gpu::Vector<HistogramConfig> {
    return &self.data.config;
  }
  
  pub fn histogram(&self) -> &gpu::Vector<u32> {
    return &self.data.histogram;
  }

  pub fn set_num_bins(& mut self, num_bins: u32) {
    if !self.data.calculated {
      let mut buff_info = gpu::BufferCreateInfo::builder()
      .gpu(0)
      .size(num_bins as usize)
      .build();
      let data: gpu::Vector<u32> = gpu::Vector::new(&self.interface, &buff_info);
      self.data.clear_bg.bind_vector("data", &data);
      self.data.calculate_bg.bind_vector("data", &data);
      self.data.histogram = data;
      
      let mapped = unsafe{self.data.config.map()};
      mapped[0].num_bins = num_bins;
      unsafe{self.data.config.unmap()};
    }
  }

  pub fn set_max_rad(& mut self, max_rad: f32) {
    let mapped = unsafe{self.data.config.map()};
    mapped[0].max_rad = max_rad;
    unsafe{self.data.config.unmap()};
  }

  pub fn set_min_rad(& mut self, min_rad: f32) {
    let mapped = unsafe{self.data.config.map()};
    mapped[0].min_rad = min_rad;
    unsafe{self.data.config.unmap()};
  }
}
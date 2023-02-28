extern crate runa;
use super::common::*;

use std::cell::RefCell;
use runa::*;

struct HistogramData {
  num_bins: u32,
  min_radiosity: f32,
  max_radiosity: f32,
  buffer: gpu::Vector<f32>,
}

struct Histogram {
  interface: * mut gpu::GraphicsData,
  data: RefCell<HistogramData>,
  data_bus: common::DataBus,
}

impl Histogram {
  fn new(interface: * mut gpu::GraphicsData) -> std::cell::RefCell<Self> {
    let histogram = to_u32_slice(include_bytes!("../target/shaders/histogram.spirv").as_ref());
    let hist = Histogram {
      interface: interface,
      data: Default::default(),
      data_bus: Default::default(),
    };

    return RefCell::new(hist);
  }
}
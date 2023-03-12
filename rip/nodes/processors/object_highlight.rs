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
/// 
#[repr(C)]
struct ObjectHighlightConfig {
  mode: u32,
}

#[derive(Default)]
struct ObjectHighlightData {
  image: Option<gpu::ImageView>,
  input: Option<gpu::ImageView>,
  scaled_down_img: gpu::Image,
  dist_image: gpu::Image,
  min_img: gpu::Image,
  max_img: gpu::Image,
  config: Option<gpu::Vector<ObjectHighlightConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
}

pub struct ObjectHighlight {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ObjectHighlightData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for ObjectHighlight {}

impl Default for ObjectHighlightConfig {
  fn default() -> Self {
      return ObjectHighlightConfig { mode: 0 }
  }
}

// Implementations specific to this node
impl ObjectHighlight {
  pub fn set_mode(& mut self, input: &String) {
    println!("Setting mode {} for node {}", input, self.name);
    let mut mode = 0;
    match input.as_str() {
      "cie_luma" => mode = 0,
      "intensity" => mode = 1,
      "i3" => mode = 2,
      _ => {},
    }

    let default_config: ObjectHighlightConfig = ObjectHighlightConfig { mode: mode };
    self.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(ObjectHighlight {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/object_highlight.spirv")).as_ref());
    let info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(raw_shader)
    .name(&info.name)
    .build();

  
    let pipeline = gpu::ComputePipeline::new(&obj.interface, &info);
    obj.data.bind_group = pipeline.bind_group();
    obj.data.pipeline = pipeline;
  
    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(1)
    .build();
  
    let default_config: ObjectHighlightConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name + "::mode"), obj.as_mut(), ObjectHighlight::set_mode);
    obj.data_bus = bus;


    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());
    return obj;
  }
}

// Base class implementations
impl RipNode for ObjectHighlight {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);

    let mut a = self.data.scaled_down_img.view();
    let mut b = self.data.max_img.view();
    let mut c = self.data.min_img.view();
    cmd.blit_views(self.data.input.as_ref().unwrap(), & mut a);
    cmd.blit_views(self.data.input.as_ref().unwrap(), & mut b);
    cmd.blit_views(self.data.input.as_ref().unwrap(), & mut c);

    cmd.bind_compute(&self.data.pipeline);
    cmd.bind(&self.data.bind_group);
    cmd.dispatch(1, 1, 1);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
    cmd.blit_views(&self.data.dist_image.view(), &mut self.data.image.as_mut().unwrap());
  }

  fn input(& mut self, image: &gpu::ImageView) {
    let c_max_img_dim = 256;
    self.data.bind_group.bind_image_view("input_tex", image);
    self.data.input = Some(image.clone());
    let scaled_width = if image.width() > c_max_img_dim { c_max_img_dim } else { image.width() };
    let scaled_height = if image.height() > c_max_img_dim { c_max_img_dim } else { image.height() };
    let info = gpu::ImageCreateInfo::builder()
    .gpu(0)
    .width(scaled_width)
    .height(scaled_height)
    .format(gpu::ImageFormat::RGBA32F)
    .build();

    self.data.scaled_down_img = gpu::Image::new(&self.interface, &info);
    self.data.dist_image = gpu::Image::new(&self.interface, &info);
    self.data.min_img = gpu::Image::new(&self.interface, &info);
    self.data.max_img = gpu::Image::new(&self.interface, &info);

    let a = vec![std::f32::MAX; self.data.dist_image.width()*self.data.dist_image.height()*4];
    self.data.dist_image.upload(&a);

    self.data.bind_group.bind_image("input_tex", &self.data.scaled_down_img);
    self.data.bind_group.bind_image("max_tex", &self.data.max_img);
    self.data.bind_group.bind_image("min_tex", &self.data.min_img);
    self.data.bind_group.bind_image("output_tex", &self.data.dist_image);
  }

  fn post_execute(& mut self, cmd: & mut gpu::CommandList) {
    cmd.synchronize();
    let a = vec![std::f32::MAX; self.data.dist_image.width()*self.data.dist_image.height()*4];
    self.data.dist_image.upload(&a);
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    self.data.image = Some(view.clone());
    self.data.bind_group.bind_image_view("output_tex", &self.data.image.as_ref().unwrap());
  }


  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "object_highlight".to_string();
  }
}
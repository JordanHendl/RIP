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

struct TransformConfig {
  transformation: [f32;16],
}

struct TransformData {
  image: Option<gpu::ImageView>,
  config: Option<gpu::Vector<TransformConfig>>,
  pipeline: gpu::ComputePipeline,
  bind_group: gpu::BindGroup,
  position: [f32; 3],
  scale: [f32; 3],
  rotation: f32,
  shear: [f32; 2],
}

impl Default for TransformData {
  fn default() -> Self {
    return TransformData { 
      image: None,
      config: None,
      pipeline: Default::default(),
      bind_group: Default::default(),
      position: [0.0, 0.0, 0.0],
      scale: [1.0, 1.0, 1.0], 
      rotation: 0.0, 
      shear: [0.0, 0.0],
    }
  }
}
pub struct Transform {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: TransformData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for Transform {}

impl Default for TransformConfig {
  fn default() -> Self {

    let t: [f32; 16] = [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0];
      return TransformConfig { transformation: t}
  }
}

// Implementations specific to this node
impl Transform {
  pub fn set_offset_x(& mut self, val: &f32) {
    println!("Setting pos x {} for node {}", val, self.name);
    self.data.position[0] = *val;
    self.update_transform();
  }

  pub fn set_offset_y(& mut self, val: &f32) {
    println!("Setting pos y {} for node {}", val, self.name);
    self.data.position[1] = *val;
    self.update_transform();
  }

  pub fn set_scale_x(& mut self, val: &f32) {
    println!("Setting scale x {} for node {}", val, self.name);
    self.data.scale[0] = *val;
    self.update_transform();
  }

  pub fn set_scale_y(& mut self, val: &f32) {
    println!("Setting scale y {} for node {}", val, self.name);
    self.data.scale[1] = *val;
    self.update_transform();
  }

  pub fn set_shear_x(& mut self, val: &f32) {
    println!("Setting shear x {} for node {}", val, self.name);
    self.data.shear[0] = *val;
    self.update_transform();
  }

  pub fn set_shear_y(& mut self, val: &f32) {
    println!("Setting shear y {} for node {}", val, self.name);
    self.data.shear[1] = *val;
    self.update_transform();
  }

  pub fn set_rotation(& mut self, val: &f32) {
    println!("Setting rotation {} for node {}", val, self.name);
    self.data.rotation = val.to_radians();
    self.update_transform();
  }

  fn update_transform(& mut self) {
    let mut t: [f32; 16] = [
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0];
    
      let r_cos = self.data.rotation.cos();
      let r_sin = self.data.rotation.sin();
      // Rotation
      t[0] = r_cos;
      t[1] = -r_sin;
      t[4] = r_sin;
      t[5] = r_cos;

      // Scale
      t[0] = self.data.scale[0];
      t[5] = self.data.scale[1];

      // Translation
      t[8] += self.data.position[0];
      t[9] += self.data.position[1];

      // Shear
      t[1] += self.data.shear[0];
      t[4] += self.data.shear[1];

      let mapped = unsafe{self.data.config.as_mut().unwrap().map()};
      mapped[0].transformation = t;
      unsafe{self.data.config.as_mut().unwrap().unmap()};
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    let mut obj = Box::new(Transform {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let raw_shader = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/transform.spirv")).as_ref());
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

    let default_config: TransformConfig = Default::default();
    obj.data.config = Some(gpu::Vector::new(&obj.interface, &buff_info));
    obj.data.config.as_mut().unwrap().upload(std::slice::from_ref(&default_config));

    let mut bus: DataBus = Default::default();
    let name = info.name.clone();
    bus.add_object_subscriber(&(name.clone() + "::off_x"), obj.as_mut(), Transform::set_offset_x);
    bus.add_object_subscriber(&(name.clone() + "::off_y"), obj.as_mut(), Transform::set_offset_y);
    bus.add_object_subscriber(&(name.clone() + "::shear_x"), obj.as_mut(), Transform::set_shear_x);
    bus.add_object_subscriber(&(name.clone() + "::shear_y"), obj.as_mut(), Transform::set_shear_y);
    bus.add_object_subscriber(&(name.clone() + "::scale_x"), obj.as_mut(), Transform::set_scale_x);
    bus.add_object_subscriber(&(name.clone() + "::scale_y"), obj.as_mut(), Transform::set_scale_y);
    bus.add_object_subscriber(&(name.clone() + "::rotation"), obj.as_mut(), Transform::set_rotation);
    obj.data_bus = bus;


    obj.data.bind_group.bind_vector("config", obj.data.config.as_ref().unwrap());
    return obj;
  }
}

// Base class implementations
impl SwsppNode for Transform {
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
    return "transform".to_string();
  }
}
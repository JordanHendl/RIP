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

struct ConnectedComponentsConfig {
}

#[derive(Default)]
struct ConnectedComponentsData {
  image: Option<gpu::ImageView>,
  config: gpu::Vector<ConnectedComponentsConfig>,
  
  indices: gpu::Vector<u32>,

  pipe_reset_cc: gpu::ComputePipeline,
  pipe_local_cc: gpu::ComputePipeline,
  pipe_boundary_cc: gpu::ComputePipeline,
  pipe_global_cc: gpu::ComputePipeline,

  reset_bg: gpu::BindGroup,
  local_bg: gpu::BindGroup,
  boundary_bg: gpu::BindGroup,
  global_bg: gpu::BindGroup,
}

pub struct ConnectedComponents {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ConnectedComponentsData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for ConnectedComponents {}

impl Default for ConnectedComponentsConfig {
  fn default() -> Self {
    ConnectedComponentsConfig {}
  }
}

// Implementations specific to this node
impl ConnectedComponents {
  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    let mut obj = Box::new(ConnectedComponents {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let shader_0 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/cc_reset.spirv")).as_ref());
    let shader_1 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/cc_local.spirv")).as_ref());
    let shader_2 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/cc_boundary_analysis.spirv")).as_ref());
    let shader_3 = common::to_u32_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/shaders/cc_global.spirv")).as_ref());

    let mut info = gpu::ComputePipelineCreateInfo::builder()
    .gpu(0)
    .shader(shader_0)
    .name(&info.name)
    .build();

    let cc_reset = gpu::ComputePipeline::new(&obj.interface, &info);
    info.shader = (&shader_1).to_vec();
    let cc_local = gpu::ComputePipeline::new(&obj.interface, &info);
    info.shader = (&shader_2).to_vec();
    let cc_boundary = gpu::ComputePipeline::new(&obj.interface, &info);
    info.shader = (&shader_3).to_vec();
    let cc_global = gpu::ComputePipeline::new(&obj.interface, &info);

    let reset_bg = cc_reset.bind_group();
    let local_bg = cc_local.bind_group();
    let boundary_bg = cc_boundary.bind_group();
    let global_bg = cc_global.bind_group();

    //let default_config: ConnectedComponentsConfig = Default::default();
    //let mut config: gpu::Vector<ConnectedComponentsConfig> = gpu::Vector::new(&obj.interface, &buff_info);
    //config.upload(std::slice::from_ref(&default_config));

    obj.data.pipe_reset_cc = cc_reset;
    obj.data.pipe_local_cc = cc_local;
    obj.data.pipe_boundary_cc = cc_boundary;
    obj.data.pipe_global_cc = cc_global;

    obj.data.reset_bg = reset_bg;
    obj.data.local_bg = local_bg;
    obj.data.boundary_bg = boundary_bg;
    obj.data.global_bg = global_bg;
    return obj;
  }
}

// Base class implementations
impl RipNode for ConnectedComponents {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    let (x, y, z) = self.data.image.as_ref().unwrap().get_compute_groups(32, 32, 1);
    cmd.bind_compute(&self.data.pipe_reset_cc);
    cmd.bind(&self.data.reset_bg);
    cmd.dispatch(x, y, z);
    cmd.vector_write_barrier(&self.data.indices);

    cmd.bind_compute(&self.data.pipe_local_cc);
    cmd.bind(&self.data.local_bg);
    cmd.dispatch(x, y, z);
    cmd.vector_write_barrier(&self.data.indices);

    // We have to run boundary analysis twice to ensure that objects are connected properly.
    cmd.bind_compute(&self.data.pipe_boundary_cc);
    cmd.bind(&self.data.boundary_bg);
    cmd.dispatch(x, y, z);
    cmd.vector_write_barrier(&self.data.indices);
    cmd.dispatch(x, y, z);
    cmd.vector_write_barrier(&self.data.indices);

    cmd.bind_compute(&self.data.pipe_global_cc);
    cmd.bind(&self.data.global_bg);
    cmd.dispatch(x, y, z);
    cmd.vector_write_barrier(&self.data.indices);
    cmd.image_write_barrier(self.data.image.as_ref().unwrap());
  }

  fn input(& mut self, image: &gpu::ImageView) {
    self.data.reset_bg.bind_image_view("input_tex", image);
    self.data.local_bg.bind_image_view("input_tex", image);
    self.data.boundary_bg.bind_image_view("input_tex", image);
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    let buff_info = gpu::BufferCreateInfo::builder()
    .gpu(0)
    .size(view.width() * view.height())
    .build();

    let indices: gpu::Vector<u32> = gpu::Vector::new(&self.interface, &buff_info);

    self.data.image = Some(view.clone());
    self.data.reset_bg.bind_vector("data", &indices);
    self.data.local_bg.bind_vector("data", &indices);
    self.data.boundary_bg.bind_vector("data", &indices);
    self.data.global_bg.bind_vector("data", &indices);
    self.data.global_bg.bind_image_view("output_tex", &self.data.image.as_ref().unwrap());
    self.data.indices = indices;
  }

  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "connected_components".to_string();
  }
}
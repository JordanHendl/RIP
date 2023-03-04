extern crate runa;
extern crate stb_image;
use std::cell::RefCell;
use std::rc::Rc;
use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;

#[derive(Default)]
struct DisplayConfig {
  fullscreen: bool,
  name: String,
}

struct DisplayData {
  window: Option<gpu::Window>,
  view: Option<gpu::ImageView>,
  num_images_blitted: u32,
}

impl Default for DisplayData {
  fn default() -> Self {
      return DisplayData { 
        window: None,
        view: None,
        num_images_blitted: 0,
      };
  }
}
pub struct Display {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: DisplayData,
  config: DisplayConfig,
  data_bus: crate::common::DataBus,
  name: String,
}
impl Display {
  fn set_fullscreen(& mut self, value: &bool) {
    self.config.fullscreen = *value;
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    println!("Creating node {} as an node!", info.name);
    let mut node = Box::new(Display {
      interface: info.interface.clone(),
      data: Default::default(),
      config: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let mut data_bus: DataBus = Default::default();
    let name = info.name.clone();
    data_bus.add_object_subscriber(&(name + "::borderless"), node.as_mut(), Display::set_fullscreen);
    node.data_bus = data_bus;

    return node;
  }
}

impl SwsppNode for Display {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    if self.data.window.is_some() && self.data.num_images_blitted < self.data.window.as_ref().unwrap().views().len().try_into().unwrap() {
      // Combo acquire into cmd, which signals some sems for cmd
      self.data.window.as_mut().unwrap().combo_next_action_into(&cmd);
      self.data.window.as_ref().unwrap().acquire();

      let views = self.data.window.as_mut().unwrap().views_mut();
      //cmd.blit_views(self.data.view.as_ref().unwrap(), & mut views[self.data.num_images_blitted as usize]);
      cmd.blit_views(self.data.view.as_ref().unwrap(), & mut views[0]);
      cmd.blit_views(self.data.view.as_ref().unwrap(), & mut views[1]);

      // Tell the command to signal sems for the window when submitted.
      cmd.combo_next_action_into_window(self.data.window.as_ref().unwrap());
      self.data.num_images_blitted += 1;
    }
  }
  
  fn post_execute(& mut self, cmd: & mut gpu::CommandList) {
    self.data.window.as_ref().unwrap().present();
    self.data.window.as_mut().unwrap().combo_next_action_into(&cmd);
    self.data.window.as_ref().unwrap().acquire();
    cmd.combo_next_action_into_window(self.data.window.as_ref().unwrap());
  }

  fn input(& mut self, image: &gpu::ImageView) {
    self.data.view = Some(image.clone());
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    if self.data.window.is_none() {
      let info = gpu::WindowCreateInfo::builder()
      .gpu(0)
      .width(view.width())
      .height(view.height())
      .num_framebuffers(1)
      .build();

      self.data.window = Some(gpu::Window::new(&self.interface, &info));
    }
  }


  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "display".to_string();
  }
}

unsafe impl Send for Display {}
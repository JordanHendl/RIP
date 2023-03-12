extern crate runa;
extern crate stb_image;
use std::cell::RefCell;
use std::rc::Rc;
use super::NodeCreateInfo;
use super::RipNode;
use super::common::*;
use runa::*;
use stb_image::image::LoadResult;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////

#[derive(Default)]
struct ImageLoadData {
  image: Option<gpu::ImageView>,
  path: String,
  loaded: bool,
  file_timestamp: Option<std::time::SystemTime>,
}

pub struct ImageLoad {
  interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ImageLoadData,
  data_bus: crate::common::DataBus,
  name: String,
}

///////////////////////////////////////////////////
/// Implementations
///////////////////////////////////////////////////

// Need send to send through threads safely
unsafe impl Send for ImageLoad {}

// Implementations specific to this node
impl ImageLoad {
  fn set_path(& mut self, file: &String) {
    self.data.path = file.clone();
    self.data.loaded = false;
  } 

  fn load_file(& mut self) {
    if !self.data.path.is_empty() {

      // Sleep for a second purely to make sure the file has no active writes. this is a cludge because I can't figure out how 
      // to make Rust wait for the file to finish writing.
      std::thread::sleep(std::time::Duration::from_millis(1000));

      let res = stb_image::image::load_with_depth(&self.data.path, 4, true);
      let devices = self.interface.borrow().devices();
      let cmd_info = gpu::CommandListCreateInfo::builder()
      .gpu(0)
      .queue_type(gpu::QueueType::Graphics)
      .build();
  
      let mut cmd = gpu::CommandList::new(&self.interface, &cmd_info);
  
      match res {
        stb_image::image::LoadResult::Error(err) => println!("RIP: Failed to load image {}!", err),
        stb_image::image::LoadResult::ImageU8(img) => {
          let info = gpu::Image::builder()
          .gpu(devices[0].id)
          .size(img.width, img.height)
          .format(gpu::ImageFormat::RGBA32F)
          .mip_count(1)
          .layers(1)
          .build();
  
          let mut gimg = gpu::Image::new(&self.interface, &info);
          let mut cpy: Vec<f32> = Vec::with_capacity(img.data.len());
          for px in &img.data {
            let new_px = (*px as f32) / (std::u8::MAX as f32);
            cpy.push(new_px);
          }
  
          gimg.upload(&cpy);
  
          cmd.begin();
          cmd.blit_views(&gimg.view(), self.data.image.as_mut().unwrap());
          cmd.end();
          cmd.submit_and_synchronize();
        },
  
        stb_image::image::LoadResult::ImageF32(img) => {
          let info = gpu::Image::builder()
          .gpu(devices[0].id)
          .size(img.width, img.height)
          .format(gpu::ImageFormat::RGBA32F)
          .mip_count(1)
          .layers(1)
          .build();
  
          let mut gimg = gpu::Image::new(&self.interface, &info);
          gimg.upload(&img.data);
  
          cmd.begin();
          cmd.blit_views(&gimg.view(), self.data.image.as_mut().unwrap());
          cmd.end();
          cmd.submit_and_synchronize();
        },
      }

      let meta = std::fs::metadata(&self.data.path);
      if let Ok(time) = meta.unwrap().modified() {
        self.data.file_timestamp = Some(time);
      }

      self.data.loaded = true;
    }
  }
  pub fn new(info: &NodeCreateInfo) -> Box<dyn RipNode + Send> {
    println!("Creating node {}!", info.name);
    let mut obj = Box::new(ImageLoad {
      interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });
    
    let mut data_bus: DataBus = Default::default();
    let name = info.name.clone();
    data_bus.add_object_subscriber(&(name + "::path"), obj.as_mut(), ImageLoad::set_path);

    obj.data_bus = data_bus;
    return obj;
  }
}

// Base class implementations
impl RipNode for ImageLoad {
  fn execute(& mut self, _cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    self.load_file();
  }

  fn input(& mut self, image: &gpu::ImageView) {
    panic!("This object is a starter and should never be called to input something!");
  }

  fn assign(& mut self, view: &gpu::ImageView) {
    self.data.image = Some(view.clone());
  }

  fn post_execute(& mut self, cmd: & mut gpu::CommandList) {
    let meta = std::fs::metadata(&self.data.path);

    if meta.is_ok() {
      if let Ok(time) = meta.unwrap().modified() {
        if *self.data.file_timestamp.as_ref().unwrap() != time {
          self.load_file();
        }
      }
    }
  }
  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "image_load".to_string();
  }
}
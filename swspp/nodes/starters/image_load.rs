extern crate runa;
extern crate stb_image;
use std::cell::RefCell;
use std::rc::Rc;
use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;
use stb_image::image::LoadResult;

///////////////////////////////////////////////////
/// Structure declarations
///////////////////////////////////////////////////

#[derive(Default)]
struct ImageLoadData {
  image: gpu::Image,
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
    println!("Image Load recieving signal to load string: {}", file);
    let res = stb_image::image::load_with_depth(file, 4, true);
    let devices = self.interface.borrow().devices();
    match res {
      stb_image::image::LoadResult::Error(err) => println!("SWSPP: Failed to load image {}!", err),
      stb_image::image::LoadResult::ImageU8(img) => {
        let info = gpu::Image::builder()
        .gpu(devices[0].id)
        .size(img.width, img.height)
        .format(gpu::ImageFormat::RGBA8)
        .mip_count(1)
        .layers(1)
        .build();

        let mut gimg = gpu::Image::new(&self.interface, &info);
        gimg.upload(&img.data);
        self.data.image = gimg;
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
        self.data.image = gimg;
      },
    }
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    println!("Creating node {} as an image load starter node!", info.name);
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
impl SwsppNode for ImageLoad {
  fn execute(& mut self, cmd: &gpu::CommandList) {
    println!("Executing Node {}", self.name);
  }

  fn input(& mut self, image: &gpu::ImageView) {
    panic!("This object is a starter and should never be called to input something!");
  }

  fn output(&self) -> gpu::ImageView {
    return self.data.image.view();
  }


  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "ImageLoad".to_string();
  }
}
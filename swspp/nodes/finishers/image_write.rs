extern crate runa;
extern crate stb_image;
use std::cell::RefCell;
use std::rc::Rc;
use super::NodeCreateInfo;
use super::SwsppNode;
use super::common::*;
use runa::*;
use stb_image_write_rust::ImageWriter::ImageWriter;

struct ImageWriteData {
  view: Option<gpu::ImageView>,
  path: String,
}

impl Default for ImageWriteData {
  fn default() -> Self {
      return ImageWriteData { 
        path: "swspp_image_output.png".to_string(),
        view: None,
      };
  }
}
pub struct ImageWrite {
  _interface: Rc<RefCell<gpu::GPUInterface>>,
  data: ImageWriteData,
  data_bus: crate::common::DataBus,
  name: String,
}
impl ImageWrite {
  fn set_path(& mut self, file: &String) {
    self.data.path = file.clone();
  }

  pub fn new(info: &NodeCreateInfo) -> Box<dyn SwsppNode + Send> {
    println!("Creating node {} as an image write finisher node!", info.name);
    let mut image_write = Box::new(ImageWrite {
      _interface: info.interface.clone(),
      data: Default::default(),
      data_bus: Default::default(),
      name: info.name.to_string(),
    });

    let mut data_bus: DataBus = Default::default();
    let name = info.name.clone();
    data_bus.add_object_subscriber(&(name + "::path"), image_write.as_mut(), ImageWrite::set_path);

    image_write.data_bus = data_bus;

    return image_write;
  }
}

impl SwsppNode for ImageWrite {
  fn execute(& mut self, cmd: & mut gpu::CommandList) {
    println!("Executing Node {}", self.name);
    cmd.end();
    cmd.submit_and_synchronize();
    cmd.begin();
    if self.data.view.is_some() {
      let view = self.data.view.as_ref().unwrap();
      let res = view.sync_get_pixels();
      match res {
        gpu::ImagePixels::ImageU8(img) => {
          let mut writer = ImageWriter::new(&self.data.path);
          writer.write_png(view.width() as i32, view.height() as i32, view.component_count() as i32, img.as_ptr());
        },
        gpu::ImagePixels::ImageF32(img) => {
          let mut writer = ImageWriter::new(&self.data.path);
          let mut converted: Vec<u8> = Vec::with_capacity(img.len());
          for idx in 0..img.len() {
            converted.push((img[idx] * (std::u8::MAX as f32)) as u8);
          }

          writer.write_png(view.width() as i32, view.height() as i32, view.component_count() as i32, converted.as_ptr());
        },
      }
      self.data.view = Default::default();
    }
  }
  
  fn input(& mut self, image: &gpu::ImageView) {
    self.data.view = Some(image.clone());
  }

  fn assign(& mut self, view: &gpu::ImageView) {
  }


  fn name(&self) -> String {
    return self.name.clone();
  }

  fn node_type(&self) -> String {
    return "ImageWrite".to_string();
  }
}

unsafe impl Send for ImageWrite {}
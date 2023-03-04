extern crate runa;
extern crate lazy_static;
use std::ffi::{CString};

mod common;
use common::*;

mod nodes;
use nodes::*;

mod gpgpu;
use gpgpu::*;

mod network;
use network::*;
#[repr(C)]
pub struct RawImageInput {
  image: * const u8,
  num_compunents: u32,
  width: u32,
  height: u32,
}

#[repr(C)]
pub struct RawFloatImageInput {
  image: * const std::os::raw::c_float,
  num_components: u32,
  width: u32,
  height: u32,
}

#[repr(C)]
pub struct VulkanImageInput {

}

#[repr(C)]
pub struct VulkanImportInfo {
  instance: u64,
  devices: * const u64, 
  num_devices: u32,
}

#[derive(Default)]
struct SwsppData {
  pipelines: std::collections::HashMap<u32, nodes::Pipeline>,
  counter: u32,
}

unsafe impl Send for SwsppData {}

use lazy_static::lazy_static;
use runa::gpu;

lazy_static! {
  static ref SWSPP_DATA: std::sync::Mutex<SwsppData> = Default::default();
}


#[no_mangle]
pub extern "C" fn swspp_create_pipeline(config: *const i8) -> u32 {
  let mut data = SWSPP_DATA.lock().expect("Unable to lock SWSPP static data!");
  let id = data.counter;
  
  let c_string = unsafe{std::ffi::CStr::from_ptr(config)};
  data.pipelines.insert(id, nodes::Pipeline::new());
  let binding = data.pipelines.get_mut(&id);
  let p = binding.unwrap();
  p.parse_json(c_string.to_str().unwrap());
  data.counter += 1;
  return id;
}

#[no_mangle]
pub extern "C" fn swspp_start_pipeline(_pipeline_id: u32) {
    
}

#[no_mangle]
pub extern "C" fn swspp_pulse() {
  let mut data = SWSPP_DATA.lock().expect("Unable to lock SWSPP static data!");
  for pipeline in & mut data.pipelines {
    pipeline.1.execute();
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
      let raw_path = concat!(env!("CARGO_MANIFEST_DIR"), "/sample_config.json");
      println!("Testing with config located at: {}", raw_path);
      let path = CString::new(raw_path).unwrap();
      let c_string = path.as_c_str().as_ptr();
      let _id = swspp_create_pipeline(c_string);

      swspp_pulse();
    }
}

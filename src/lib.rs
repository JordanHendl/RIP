extern crate runa;
extern crate lazy_static;
use std::ffi::{c_float, CString};

mod common;
use common::*;

mod nodes;
use nodes::*;

#[repr(C)]
pub struct RawImageInput {
  image: * const u8,
  num_compunents: u32,
  width: u32,
  height: u32,
}

#[repr(C)]
pub struct RawFloatImageInput {
  image: * const c_float,
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

use lazy_static::lazy_static;

lazy_static! {
  static ref SWSPP_DATA: std::sync::Mutex<SwsppData> = Default::default();
}

#[no_mangle]
pub extern "C" fn swspp_create_pipeline(config: *const i8) -> u32 {
  let mut data = SWSPP_DATA.lock().expect("Unable to lock SWSPP static data!");
  let id = data.counter;
  
  let c_string = unsafe{std::ffi::CStr::from_ptr(config)};
  data.pipelines.insert(id, nodes::Pipeline::new(c_string.to_str().unwrap()));

  data.counter += 1;
  return id;
}

#[no_mangle]
pub extern "C" fn swspp_start_pipeline(_pipeline_id: u32) {
    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}

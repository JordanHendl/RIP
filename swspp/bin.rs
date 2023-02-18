extern crate swspp;
use swspp::*;
use std::ffi::{CString};
fn main() {
  let string = std::env::args().nth(1).expect("No Configuration path given! Aborting.");
  let path = string.as_str();
  println!("Testing with config located at: {}", path);
  let path = CString::new(path).unwrap();
  let c_string = path.as_c_str().as_ptr();
  let _id = swspp_create_pipeline(c_string);
  swspp_pulse();
}
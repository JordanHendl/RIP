extern crate rip;
use rip::*;
use std::ffi::{CString};
fn main() {
  let string = std::env::args().nth(1).expect("No Configuration path given! Aborting.");
  let path = string.as_str();
  println!("Testing with config located at: {}", path);
  let path = CString::new(path).unwrap();
  let c_string = path.as_c_str().as_ptr();
  let _id = rip_create_pipeline(c_string);


  let args = std::env::args().nth(2);
  if args.is_some() {
    let str = args.unwrap().clone();
    if str.eq(&"--repeat".to_string()) {
      while rip_should_run() {
        rip_pulse();
      }
    } else {
      rip_pulse();  
    }
  } else {
    rip_pulse();
  }
}
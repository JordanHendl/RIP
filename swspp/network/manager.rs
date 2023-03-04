use protobuf::{EnumOrUnknown, Message};

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

use response::*;

struct Manager {
  ctx: zmq::Context,
  socket: zmq::Socket,
}

impl Manager {
  pub fn new() -> Self {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();
    socket.bind("tcp://*:5555").expect("Failed to bind to ZMQ Port!");

    return Manager { 
      ctx: ctx,
      socket: socket,
    };
  }
}
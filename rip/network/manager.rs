
use protobuf::{EnumOrUnknown, Message};
use crate::message::*;

pub struct Manager {
  ctx: zmq::Context,
  socket: zmq::Socket,
}

impl Manager {
  pub fn new() -> Self {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://127.0.0.1:5555").expect("Failed to bind to ZMQ Port!");

    return Manager { 
      ctx: ctx,
      socket: socket,
    };
  }

  pub fn receive_message(& mut self) -> Option<zmq::Message> {
    let mut message = zmq::Message::new();
    let res = self.socket.recv(&mut message, zmq::DONTWAIT);

    match res {
      Ok(..) => Some(message),
      Err(..) => None,
    }
  }

  pub fn send_response(& mut self, response: crate::message::Response) {
    let bytes = response.write_to_bytes();
    self.socket.send(bytes.as_ref().unwrap(), Default::default()).expect("Error sending message!");
  }
}
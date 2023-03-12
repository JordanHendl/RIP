
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
    return Manager { 
      ctx: ctx,
      socket: socket,
    };
  }

  pub fn bind(& mut self, ip: &String, port: &String) {

    let mut str = "tcp://".to_string();
    str.push_str(ip);
    str.push_str(":");
    str.push_str(port);

    println!("rip-- binding to ip {} and port {}", ip, port);
    self.socket.bind(&str.as_str()).expect("Failed to bind to ZMQ Port!");
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
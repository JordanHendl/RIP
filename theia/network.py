import zmq
import message_pb2
import numpy
class RipNetwork:
  def __init__(self):
    self.ctx = zmq.Context()
    self.socket = self.ctx.socket(zmq.REQ)

  def connect(self, ip, port):
    str = "tcp://"
    str += ip
    str += ":"
    str += port
    print("Connecting to ", str)
    self.socket.connect(str)

  def get_json(self):
    msg = message_pb2.Request()
    msg.node_name = "N/A"
    msg.request_type = message_pb2.RequestType.Configuration
    bytes = msg.SerializeToString()
    self.socket.send(bytes)
    response = self.socket.poll(timeout=3000)
    if response == 0:
      print("Timed out communication with rip!")
      return ""
    else:
      msg = self.socket.recv()
      pb_response = message_pb2.Response()
      pb_response.ParseFromString(msg)
      config_response = pb_response.config_response
      if config_response:
        print("received json from rip: {}", str(config_response.json))
        return config_response.json
      else:
        return ""
      
  def get_image(self, node_name):
    msg = message_pb2.Request()
    msg.node_name = node_name
    msg.request_type = message_pb2.RequestType.Image
    bytes = msg.SerializeToString()
    self.socket.send(bytes)
    response = self.socket.poll(timeout=3000)
    if response == 0:
      print("Timed out communication with rip!")
      return ""
    else:
      msg = self.socket.recv()
      pb_response = message_pb2.Response()
      pb_response.ParseFromString(msg)
      if pb_response.image_response:
        return (pb_response.image_response.width, pb_response.image_response.height, pb_response.image_response.image)
      else:
        return (None, None, None)

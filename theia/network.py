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

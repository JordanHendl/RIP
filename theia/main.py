import message_pb2
import zmq
from matplotlib import pyplot as plt
import numpy

def main(): 
  port = "5555"
  ctx = zmq.Context()
  socket = ctx.socket(zmq.REQ)
  socket.connect("tcp://127.0.0.1:5555")

  print(zmq.pyzmq_version())
  msg = message_pb2.Request()

  msg.node_name = "pattern_2"
  msg.request_type = message_pb2.RequestType.Image
  bytes = msg.SerializeToString()

  for request in range(1):
    print(f"Sending request {request}")

    socket.send(bytes)

    print("Waiting on response!")
    response = socket.poll(timeout=3000)
    if response == 0:
      print("Timeout! No response received! Sad :(")
      pass
    else:
      print(f"Received reply! {request}")
      msg = socket.recv()
      pb_response = message_pb2.Response()
      pb_response.ParseFromString(msg)
      img = pb_response.image_response.image
      reshaped = numpy.reshape(img, (pb_response.image_response.height,
                                     pb_response.image_response.width,
                                     pb_response.image_response.num_channels))
      plt.imshow(reshaped)
      plt.show()
if __name__ == "__main__":
  main()
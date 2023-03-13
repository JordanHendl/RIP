from network import RipNetwork
from PyQt6 import QtWidgets
from PyQt6 import QtGui
from PyQt6.QtWidgets import QApplication, QLabel
import json
import numpy
from matplotlib import pyplot as plt
class ConnectTab(QtWidgets.QDialog):
  def __init__(self, window, cb):
    super().__init__(window)
    self.setWindowTitle("Connect to runtime")
    self.setGeometry(0, 0, 300, 150)
    self.layout = QtWidgets.QVBoxLayout()
    self.cb = cb
    self.ip_label = QtWidgets.QLabel("IP:")
    self.ip_input = QtWidgets.QLineEdit("127.0.0.1")
    self.port_label = QtWidgets.QLabel("Port:")
    self.port_input = QtWidgets.QLineEdit("5555")
    self.confirm = QtWidgets.QPushButton("Connect", self)
    self.confirm.setToolTip("Attempt to connect to rip runtime using ip and port")
    self.confirm.clicked.connect(self.on_click)

    self.layout.addWidget(self.ip_label)
    self.layout.addWidget(self.ip_input)
    self.layout.addWidget(self.port_label)
    self.layout.addWidget(self.port_input)
    self.layout.addWidget(self.confirm)
    self.setLayout(self.layout)

  def on_click(self):
    self.cb(self.ip_input.text(), self.port_input.text())
    self.close()

class NodeProperties(QtWidgets.QWidget):
  def __init__(self, window, item_cb):
    super().__init__(window)
    self.layout = QtWidgets.QVBoxLayout()
    self.nodes = QtWidgets.QComboBox(self)
    self.nodes.currentTextChanged.connect(item_cb)
    self.layout.addWidget(self.nodes)
    self.layout.addStretch()

  def update_json(self, json_data):
    self.nodes.clear()
    starters = json_data["starters"]
    imgproc = json_data["imgproc"]
    finishers = json_data["finishers"]

    for s in starters:
      self.nodes.addItem(s)

    for i in imgproc:
      self.nodes.addItem(i)

    for f in finishers:
      self.nodes.addItem(f)

class MainArea(QtWidgets.QWidget):
  def __init__(self, window, item_clicked_cb):
    super().__init__(window)
    self.image = QtGui.QPixmap("./tulips.png")
    self.label = QLabel("Image Output")
    self.grid = QtWidgets.QGridLayout()

    self.node_properties = NodeProperties(window, item_clicked_cb)
    self.label.setPixmap(self.image)
    self.grid.addWidget(self.label,0,1)
    self.grid.addWidget(self.node_properties,1,0)
    self.setLayout(self.grid)
    self.label.show()

  def handle_json(self, json):
    self.node_properties.update_json(json)

  def update_image(self, width, height, pixels):
    qimage = QtGui.QImage(pixels, width, height, QtGui.QImage.Format.Format_RGBA32FPx4)
    self.image = QtGui.QPixmap(qimage)
    self.label.setPixmap(self.image)

class TheiaMainWindow(QtWidgets.QMainWindow): 
  def __init__(self):
    super().__init__()
    c_window_width = 1024
    c_window_height = 718
    self.setGeometry(0, 0, c_window_width, c_window_height)
    self.setWindowTitle("theia")

    # Custom Widgets
    self.network = RipNetwork()
    self.connect_tab = ConnectTab(self, self.initialize_connection)

    # Menu widgets
    self.menu_bar = self.menuBar()
    self.file_menu = self.menu_bar.addMenu('&File')

    self.connect_to_rip = QtGui.QAction("&Connect")
    self.connect_to_rip.setStatusTip("Connect to an instance of 'rip'")
    self.connect_to_rip.triggered.connect(self.show_connect_tab)
    self.file_menu.addAction(self.connect_to_rip)

    # Central widget
    self.tabs = QtWidgets.QTabWidget()
    self.central_widget = MainArea(self, self.node_clicked)
    self.tabs.addTab(self.central_widget, "Pipeline Info")
    self.setCentralWidget(self.tabs)
    self.show()

  def parse_json(self):
    if self.json != "": 
      self.parsed = json.loads(self.json)
      self.central_widget.handle_json(self.parsed)

  def initialize_connection(self, ip, port):
    self.network.connect(ip, port)
    self.json = self.network.get_json()
    if self.json == "":
      error_dialog = QtWidgets.QErrorMessage()
      error_dialog.showMessage('Failed to connect! Timeout occured!')
      error_dialog.exec()

    self.parse_json()

  def node_clicked(self, item):
    (width, height, img) = self.network.get_image(item)

    if img is not None:
      img = numpy.asarray(img)
      img = img.astype(numpy.float32)
      reshaped = numpy.reshape(img, (height,
                                     width,
                                     4))
      
      self.central_widget.update_image(width, height, reshaped)

  def show_connect_tab(self):
    self.connect_tab.show()

def run(argv):
  app = QtWidgets.QApplication(argv)
  gui = TheiaMainWindow()
  return app.exec()
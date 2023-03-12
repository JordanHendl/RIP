from PyQt5 import QtWidgets
from PyQt5 import QtGui
from PyQt5.QtWidgets import QApplication, QLabel

class ConnectTab(QtWidgets.QDialog):
  def __init__(self, window):
    super().__init__(window)
    self.setWindowTitle("Connect to runtime")
    self.setGeometry(0, 0, 300, 150)
    self.layout = QtWidgets.QVBoxLayout()

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
    print("clicked!")

class MainAreaTab(QtWidgets.QWidget):
  def __init__(self, window):
    super().__init__(window)
    self.image = QtGui.QPixmap()
    self.label = QLabel()
    self.label.setPixmap(self.image)
    self.grid = QtWidgets.QGridLayout()
    self.grid.addWidget(self.label,1,1)
    self.setLayout(self.grid)

class TheiaMainWindow(QtWidgets.QMainWindow):
  def __init__(self):
    super().__init__()
    c_window_width = 1024
    c_window_height = 718
    self.setGeometry(0, 0, c_window_width, c_window_height)
    self.setWindowTitle("theia")

    # Custom Widgets
    self.connect_tab = ConnectTab(self)

    # Menu widgets
    self.menu_bar = self.menuBar()
    self.file_menu = self.menu_bar.addMenu('&File')

    self.connect_to_rip = QtWidgets.QAction("&Connect")
    self.connect_to_rip.setStatusTip("Connect to an instance of 'rip'")
    self.connect_to_rip.triggered.connect(self.show_connect_tab)
    self.file_menu.addAction(self.connect_to_rip)

    # Central widget
    self.central_widget = MainAreaTab(self)
    self.setCentralWidget(self.central_widget)
    self.show()

  def show_connect_tab(self):
    print("lmao")
    self.connect_tab.show()



def run(argv):
  app = QtWidgets.QApplication(argv)
  gui = TheiaMainWindow()
  return app.exec_()
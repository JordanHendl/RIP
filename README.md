(R)ust (I)mage (P)rocessor

Node-based GPU image processor using Vulkan.

See sample configs for example modes of operation, see docs directory for node documentations. 

Node Types:

There are starter, imgproc, and finisher nodes.

Starter nodes can generate images. They do not receive any images.

Finisher nodes consume images. They may not generate any.

ImgProc nodes both consume and generate images. This may be multiple input, or multiple output.

| Name of sample configuration   | Output |
| ---------------------------    | ------ |
| arithmetic_multiple_input.json | ![](https://i.imgur.com/MvroQI8.png) |
| chroma_key.json | ![](https://i.imgur.com/YJ1sCgR.png) |
| circle_pattern.json | ![](https://i.imgur.com/0cVE3fk.png) |
| color_spaces.json | ![](https://i.imgur.com/UFEJpIE.png) |
| crop.json | ![](https://i.imgur.com/jQzUSVS.png) |
| reflected_x_axis.json | ![](https://i.imgur.com/vPTbfPn.png) |
| reflected_y_axis.json | ![](https://i.imgur.com/KS9b268.png) |
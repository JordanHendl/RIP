(R)ust (I)mage (P)rocessor

Node-based GPU image processor using Vulkan.

See sample configs for example modes of operation, see docs directory for node documentations. 

Node Types:

There are starter, imgproc, and finisher nodes.

Starter nodes can generate images. They do not receive any images.

Finisher nodes consume images. They may not generate any.

ImgProc nodes both consume and generate images. This may be multiple input, or multiple output.
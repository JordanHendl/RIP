#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable
#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 32 
#define BLOCK_SIZE_Z 1 
#define BLOCK_THREAD_COUNT BLOCK_SIZE_X*BLOCK_SIZE_Y*BLOCK_SIZE_Z

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in; 

layout(binding = 0, rgba32f) coherent restrict readonly uniform image2D input_tex;

layout(binding = 1) writeonly buffer Indices {
  uint indices[];
} data;

void main() {
  const ivec2 dim = imageSize(input_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const uint gid = ix + (iy * uint(dim.x));
  data.indices[gid] = 0;
}
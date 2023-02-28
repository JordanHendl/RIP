#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable
#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 1 
#define BLOCK_SIZE_Z 1 

layout( local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z ) in ; 
layout(binding = 0) writeonly buffer HistogramData {
  uint histogram[];
} data;

void main() {
  uint global_id = gl_GlobalInvocationID.x;
  data.histogram[global_id] = 0;  
}
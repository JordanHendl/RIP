#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 32 
#define BLOCK_SIZE_Z 1 
#define MAX_BINS 1024

layout( local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z ) in ; 
layout( binding = 0, rgba32f ) coherent restrict readonly  uniform image2D input_tex;

layout(binding = 1) uniform HistogramConfig {
  uint num_bins;
  float min_rad;
  float max_rad;
} config;

layout(binding = 2) writeonly buffer HistogramData {
  uint histogram[];
} data;

void main() {
  const ivec2 coords = ivec2(gl_GlobalInvocationID.xy);
  const ivec2 dim = imageSize(input_tex);
  uint global_id = gl_GlobalInvocationID.x * (gl_GlobalInvocationID.y * dim.x);
  
  const vec4 radiance = imageLoad(input_tex, coords);
  const float pix = radiance.r;
  uint bin = clamp(uint((pix - config.min_rad)/(config.max_rad-config.min_rad) * config.num_bins), 0, config.num_bins-1);
  if(radiance.a > 0) {
    atomicAdd(data.histogram[bin], 1);  
  }
}
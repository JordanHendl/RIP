#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 1 
#define BLOCK_SIZE_Z 1 
#define MAX_BINS 1024

layout( local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z ) in ; 
layout( binding = 0, rgba32f ) coherent restrict readonly uniform image2D input_tex;

layout(binding = 2) uniform HistogramConfig {
  uint num_bins;
  float min_rad;
  float max_rad;
} hist_config;

layout(binding = 3) readonly buffer HistogramData {
  uint histogram[];
} hist_data;

layout(binding = 4) writeonly buffer TonemapData {
  float tonecurve[];
} data;

layout(binding = 5) readonly buffer TonemapConfig {
  uint mode;
} config;

float normalized() {
  const ivec2 dim = imageSize(input_tex);
  uint global_id = gl_GlobalInvocationID.x;
  uint scan = 0;
  for(uint idx = 0; idx < global_id; idx++) {
    scan += hist_data.histogram[idx];
  }

  return float(scan) / (dim.x * dim.y);
}

void main() {
  uint global_id = gl_GlobalInvocationID.x;
  const uint mode = config.mode;
  float tone = 0.0;
  switch(mode) {
    default: tone = normalized();
  };

  data.tonecurve[global_id] = tone;
}
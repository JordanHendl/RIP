#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 32 
#define BLOCK_SIZE_Z 1 
#define MAX_BINS 1024

layout( local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z ) in ; 
layout( binding = 0, rgba32f ) coherent restrict readonly uniform image2D input_tex;
layout( binding = 1, rgba32f ) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform HistogramConfig {
  uint num_bins;
  float min_rad;
  float max_rad;
} hist_config;

layout(binding = 3) readonly buffer HistogramData {
  uint histogram[];
} hist_data;

layout(binding = 4) readonly buffer TonemapConfig {
  uint mode;
} config;

float normalized(float radiance) {
  const ivec2 dim = imageSize(input_tex);
  uint global_id = gl_GlobalInvocationID.x * (gl_GlobalInvocationID.y * dim.x);
  uint bin = clamp(uint((radiance - hist_config.min_rad)/(hist_config.max_rad-hist_config.min_rad) * hist_config.num_bins), 0, hist_config.num_bins-1);

  uint scan = 0;
  for(uint idx = 0; idx < bin; idx++) {
    scan += hist_data.histogram[idx];
  }

  float tone = float(scan) / (dim.x * dim.y);
  return radiance * tone;
}

void main() {
  const uint mode = config.mode;
  const ivec2 coords = ivec2(gl_GlobalInvocationID.xy);  
  const vec4 radiance = imageLoad(input_tex, coords);
  const float pix = radiance.r;
  float intensity = 0.0;
  switch(mode) {
    default: intensity = normalized(pix);
  }

  imageStore(output_tex, coords, vec4(vec3(intensity), 1.0));
}
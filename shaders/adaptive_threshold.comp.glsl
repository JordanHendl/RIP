#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform ThresholdConfig {
  uint radius;
  uint mode;
} config;


struct LocalStats {
  float mean;
  float min;
  float max;
  float stddev;
};

LocalStats make_stats() {
  LocalStats s;
  s.mean = 0.0f;
  s.stddev = 0.0f;
  s.min = 500.0f;
  s.max = -500.0f;
  return s;
}

vec3 thresh_stddev() {
  const ivec2 tex_coord = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  const ivec2 dim = imageSize(input_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const int radius = int(config.radius);

  LocalStats stats = make_stats();
  for(int u = -radius; u <= radius; u++) {
    for(int v = -radius; v <= radius; v++) {
      const ivec2 coord = ivec2(ix + u, iy + v);
      const vec4 color = imageLoad(input_tex, coord);
      const float pixel = color.r;
      stats.mean += pixel;
      stats.min = min(pixel, stats.min);      
      stats.max = max(pixel, stats.max);
    }
  }

  int width = (radius + radius + 1);
  stats.mean = (stats.mean / float(width * width));

  for(int u = -radius; u <= radius; u++) {
    for(int v = -radius; v <= radius; v++) {
      const ivec2 coord = ivec2(ix + u, iy + v);
      const vec4 color = imageLoad(input_tex, coord);
      const float pixel = color.r;
      float tmp = pixel - stats.mean;
      stats.stddev += (tmp * tmp);
    }
  }

  stats.stddev /= float(width * width);

  const vec4 color = imageLoad(input_tex, tex_coord);
  return color.r > (stats.mean - stats.stddev) ? vec3(1.0f) : vec3(0.0f);
}

vec3 thresh_constant() {
  const ivec2 tex_coord = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  const ivec2 dim = imageSize(input_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const int radius = int(config.radius);

  LocalStats stats = make_stats();
  for(int u = -radius; u <= radius; u++) {
    for(int v = -radius; v <= radius; v++) {
      const ivec2 coord = ivec2(ix + u, iy + v);
      const vec4 color = imageLoad(input_tex, coord);
      const float pixel = color.r;
      stats.mean += pixel;
      stats.min = min(pixel, stats.min);      
      stats.max = max(pixel, stats.max);
    }
  }

  int width = (radius + radius + 1);
  stats.mean = (stats.mean / float(width * width));

  const float constant_value = 0.005f;
  const vec4 color = imageLoad(input_tex, tex_coord);
  return color.r > (stats.mean - constant_value) ? vec3(1.0f) : vec3(0.0f);
}

void main() {
  const uint mode = config.mode;
  const ivec2 tex_coords = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec3 out_color;
  switch(mode) {
    case 0: out_color = thresh_constant(); break;
    case 1: out_color = thresh_stddev(); break;
    default: out_color = thresh_stddev(); break;
  }

  imageStore(output_tex, tex_coords, vec4(out_color, 1));
}

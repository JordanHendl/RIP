#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform PatternConfig {
  uint mode;
} config;

vec4 circle_pattern() {
  const ivec2 dim = imageSize(output_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const float radius = dim.x / 2.0;

  const ivec2 center = ivec2(dim.x / 2, dim.y / 2);

  float dist = sqrt(pow(float(ix) - float(center.x), 2) + pow(float(iy) - float(center.y), 2));
  if(dist < radius) {
    return vec4(1.0, 1.0, 1.0, 1.0);
  }

  return vec4(0.0, 0.0, 0.0, 0.0);
}

vec4 horizontal_color_bar_pattern() {
  const ivec2 dim = imageSize(output_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;

  const uint num_bars = 8;
  const uint bar_size = dim.y / num_bars;
  vec4 colors[num_bars];

  colors[0] = vec4(1.0, 0.0, 0.0, 1.0);
  colors[1] = vec4(0.7, 0.3, 0.0, 1.0);
  colors[2] = vec4(0.5, 0.5, 0.0, 1.0);
  colors[3] = vec4(0.3, 0.7, 0.0, 1.0);
  colors[4] = vec4(0.0, 1.0, 0.0, 1.0);
  colors[5] = vec4(0.0, 0.7, 0.3, 1.0);
  colors[6] = vec4(0.0, 0.5, 0.5, 1.0);
  colors[7] = vec4(0.0, 0.3, 0.7, 1.0);

  uint idx = iy / bar_size;
  if(idx >= 8) idx = 0;
  return colors[idx];
}

vec4 horizontal_bar_pattern() {
  const ivec2 dim = imageSize(output_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;

  const uint num_bars = 8;
  const uint bar_size = dim.y / num_bars;
  if(iy % bar_size < bar_size / 2) {
    return vec4(1,1,1,1);
  } else {
    return vec4(0,0,0,0);
  }
}

vec4 gray_pattern() {
  return vec4(0.5, 0.5, 0.5, 1.0);
}

vec4 white_pattern() {
  return vec4(1.0, 1.0, 1.0, 1.0);
}

vec4 blank_pattern() {
  return vec4(0.0, 0.0, 0.0, 0.0);
}

void main() {
  const uint mode = config.mode;
  const ivec2 tex_coords = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 out_color;
  switch(mode) {
    case 5: out_color = circle_pattern(); break;
    case 4: out_color = gray_pattern(); break;
    case 3: out_color = white_pattern(); break;
    case 2: out_color = horizontal_color_bar_pattern(); break;
    case 1: out_color = horizontal_bar_pattern(); break;
    case 0: out_color = blank_pattern(); break;
    default: out_color = blank_pattern(); break;
  }

  imageStore(output_tex, tex_coords, out_color);
}

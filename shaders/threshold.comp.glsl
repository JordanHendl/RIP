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
  uint mode;
  float constant;
} config;

vec3 thresh_constant() {
  const ivec2 dim = imageSize(input_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const ivec2 tex_coord = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  const vec4 color = imageLoad(input_tex, tex_coord);
  
  return color.r > (config.constant) ? vec3(1.0f) : vec3(0.0f);
}

void main() {
  const uint mode = config.mode;
  const ivec2 tex_coords = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec3 out_color;
  switch(mode) {
    case 0: out_color = thresh_constant(); break;
    default: out_color = thresh_constant(); break;
  }

  imageStore(output_tex, tex_coords, vec4(out_color, 1));
}

#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform CropConfig {
  uint top;
  uint left;
  uint bottom;
  uint right;
} config;

bool is_in_bounds() {
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;

  if(iy > config.top && iy < config.bottom && ix > config.left && ix < config.right) {
    return true;
  } 

  return false;
}

void main() {
  const ivec2 tex_coords = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 color = imageLoad(input_tex, tex_coords);
  vec4 blank = vec4(0.0);

  if(is_in_bounds()) {
    imageStore(output_tex, tex_coords, color);  
  } else {
    imageStore(output_tex, tex_coords, blank);
  }
}

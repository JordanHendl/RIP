#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

#include "include/color_space.glsl"

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 2, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 3) uniform ColorSpaceConverterConfig {
  uint src_mode;
  uint dst_mode;
} config;

void main() {
  const ivec2 tex_coords = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 color = imageLoad(input_tex, tex_coords);

  vec3 out_color = vec3(0, 0, 0);
  switch(config.src_mode) {
    case 0: // RGB Color space
      switch(config.dst_mode) {
        case 0: out_color = color.rgb; break; // Same color space, no conversion
        case 1: out_color = rgb2ycbcr(color.rgb); break; // RGB -> YCbCr
        case 2: out_color = rgb2hsv(color.rgb); break; // RGB -> HSV
    } break;
    case 1:  // YCbCr
      switch(config.dst_mode) {
        case 0: out_color = ycbcr2rgb(color.rgb); break; // YCbCr -> RGB
        case 1: out_color = color.rgb; break; // Same color space, no conversion
        case 2: out_color = rgb2hsv(ycbcr2rgb(color.rgb)); break; // YCbCr -> RGB -> HSV
    } break;
    case 2: // HSV
      switch(config.dst_mode) {
        case 0: out_color = hsv2rgb(color.rgb); break; // HSV -> RGB
        case 1: out_color = ycbcr2rgb(hsv2rgb(color.rgb)); break; // HSV -> RGB -> YCbCr
        case 2: out_color = color.rgb; break; // Same color space, no conversion
      }
  }
  imageStore(output_tex, tex_coords, vec4(out_color, 1.0)); 
}

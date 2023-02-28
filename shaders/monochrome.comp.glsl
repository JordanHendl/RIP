#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform MonochromeConfig {
  uint mode;
} config;

// This is formula to calculate relative luminance. It converts RGB -> Photometric luminance
// The coefficients are based on the CIE color matching functions and relevant standard chromaticities of RGB.
// These coefficients are for for sRGB.
// See https://en.wikipedia.org/wiki/Luma_(video)
vec3 convert_CIELuminance(vec3 in_color) {
  const float intensity  = ( 0.2126 * in_color.r + 0.7152 * in_color.g + 0.0722 * in_color.b);
  return vec3(intensity);
}

vec3 convert_normalized(vec3 in_color) {
  const float intensity = (in_color.r + in_color.g + in_color.b) / 3.0;
  return vec3(intensity);
}

vec3 convert_i3(vec3 in_color) {
  const float intensity = (((2.0 * in_color.g) - in_color.b - in_color.r) / 4.0);
  return vec3(intensity);
}

void main() {
  const uint mode = config.mode;

  const ivec2 tex_coords = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 color = imageLoad(input_tex, tex_coords);
  vec3 out_color;

  switch(mode) {
    case 0: out_color = convert_CIELuminance(color.rgb); break;
    case 1: out_color = convert_normalized(color.rgb); break;
    case 2: out_color = convert_i3(color.rgb); break;
    default: out_color = convert_normalized(color.rgb); break;
  }

  imageStore(output_tex, tex_coords, vec4(out_color, 1));
}

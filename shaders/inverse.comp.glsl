#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

void main() {
  const ivec2 tex_coords = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 color = imageLoad( input_tex, tex_coords );

  color.r = 1.0 - color.r;
  color.g = 1.0 - color.g;
  color.b = 1.0 - color.b;
  imageStore( output_tex, tex_coords, color );
}

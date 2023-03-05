#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 2) uniform TransformConfig {
  mat4 transform;
} config;

void main() {
  const ivec2 tex_coord = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  const vec4 color = imageLoad(input_tex, tex_coord);

  vec4 coords_to_transform = vec4(tex_coord.x, tex_coord.y, 1 , 1);

  vec4 new_location = config.transform * coords_to_transform;
  const ivec2 new_tex_coord = ivec2(int(new_location.x), int(new_location.y));
  imageStore(output_tex, new_tex_coord, color);
}

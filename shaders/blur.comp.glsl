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
  const int radius = 3;
  const ivec2 center = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  int xcenter = int(gl_GlobalInvocationID.x);
  int ycenter = int(gl_GlobalInvocationID.y);
  vec4 out_color = vec4(0, 0, 0, 0);
  for(int u = -radius; u <= radius; u++) {
    for(int v = -radius; v <= radius; v++) {
      const ivec2 coord = ivec2(xcenter + u, ycenter + v);
      const vec4 color = imageLoad(input_tex, coord);
      out_color += color;
    }
  }

  int width = (radius + radius + 1);
  out_color = (out_color / (vec4(float(width * width))));
  out_color.a = 1.0;
  imageStore(output_tex, center, out_color);
}

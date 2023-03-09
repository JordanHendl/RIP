#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32
#define BLOCK_SIZE_Y 32
#define BLOCK_SIZE_Z 1

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

#include "include/color_space.glsl"

layout(binding = 0, rgba32f) coherent restrict readonly  uniform image2D input_tex_0;
layout(binding = 1, rgba32f) coherent restrict readonly  uniform image2D input_tex_1;
layout(binding = 2, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout(binding = 3) uniform CropConfig {
  vec3 rgb;
  float lower_range;
  float higher_range;
} config;

float color_dist(float Cb_p, float Cr_p, float Cb_key, float Cr_key, float tola, float tolb) {
   /*decides if a color is close to the specified hue*/
   float temp = sqrt((Cb_key-Cb_p)*(Cb_key-Cb_p)+(Cr_key-Cr_p)*(Cr_key-Cr_p));
   if (temp < tola) {return (0.0);}
   if (temp < tolb) {return ((temp-tola)/(tolb-tola));}
   return (1.0);
}

/** Algorithm referenced: http://gc-films.com/chromakey.html
* Gotten from sources on the wikipedia page for Chroma key https://en.wikipedia.org/wiki/Chroma_key
* I have no clue why wikipedia linked to this random religious guy's article, but his chroma key algorithm does work pretty well....
*/
void main() {
  const ivec2 tex_coords = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
  vec4 color = imageLoad(input_tex_0, tex_coords);
  vec4 color2 = imageLoad(input_tex_1, tex_coords);
  vec3 color_ycbcr = rgb2ycbcr(color.rgb) * vec3(256.0);
  vec3 key_ycbcr = rgb2ycbcr(config.rgb) * vec3(256.0);
  vec3 color_mask = config.rgb * vec3(256.0);
  color = color * vec4(256.0);
  color2 = color2 * vec4(256.0);

  float tola = config.lower_range;
  float tolb = config.higher_range;
  float mask = 1.0 - color_dist(color_ycbcr.g, color_ycbcr.b, key_ycbcr.g, key_ycbcr.b, tola, tolb);

  vec3 out_color = vec3(max(color.r - (mask*color_mask.r), 0) + (mask*color2.r),
                        max(color.g - (mask*color_mask.g), 0) + (mask*color2.g),
                        max(color.b - (mask*color_mask.b), 0) + (mask*color2.b));

  out_color = out_color / vec3(256.0);
  imageStore(output_tex, tex_coords, vec4(out_color, 1.0)); 
}

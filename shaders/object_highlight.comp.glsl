#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 1
#define BLOCK_SIZE_Y 1
#define BLOCK_SIZE_Z 1
layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in;

layout(binding = 0, rgba32f) coherent restrict readonly uniform image2D input_tex;
layout(binding = 1, rgba32f) coherent restrict uniform image2D max_tex;
layout(binding = 2, rgba32f) coherent restrict uniform image2D min_tex;
layout(binding = 3, rgba32f) coherent restrict uniform image2D output_tex;

#define pixel(x, y, img) imageLoad(img, ivec2(x, y)).r

layout(binding = 4) uniform ObjectHighlightConfig {
  uint mode;
} config;

float cost_function(float uy, float ix, float ly) {
  return max(uy, ix) - min(ly, ix);
}

void inverse_raster_scan() {
  const ivec2 dim = imageSize(input_tex);
  const uint size = uint(dim.x * dim.y);

  for(int x = dim.x - 1; x >= 0; x--) {
    for(int y = dim.y - 1; y >= 0; y--) {
      float ix = pixel(x, y, input_tex);
      float uy_right = pixel(x+1, y, max_tex);
      float uy_down = pixel(x, y+1, max_tex);
      float ly_right = pixel(x+1, y, min_tex);
      float ly_down = pixel(x, y+1, min_tex);
      float d = pixel(x, y, output_tex);
      
      if(y == dim.y - 1) ly_down = 1.0;
      if(x == dim.x - 1) ly_right = 1.0;
      float d1 = cost_function(uy_right, ix, ly_right);
      if(d1 < d) {
        d = d1;
        imageStore(output_tex, ivec2(x, y), vec4(vec3(d1), 1));
        imageStore(max_tex, ivec2(x, y), vec4(vec3(max(uy_right, ix)), 1));
        imageStore(min_tex, ivec2(x, y), vec4(vec3(min(ly_right, ix)), 1));
      }

      float d2 = cost_function(uy_down, ix, ly_down);
      if(d2 < d) {
        d = d2;
        imageStore(output_tex, ivec2(x, y), vec4(vec3(d2), 1));
        imageStore(max_tex, ivec2(x, y), vec4(vec3(max(uy_down, ix)), 1));
        imageStore(min_tex, ivec2(x, y), vec4(vec3(min(ly_down, ix)), 1));
      }
    }
  }  
}

void raster_scan() {
  const ivec2 dim = imageSize(input_tex);
  const uint size = uint(dim.x * dim.y);

  for(int x = 0; x < dim.x; x++) {
    for(int y = 0; y < dim.y; y++) {
      float ix = pixel(x, y, input_tex);
      float uy_left = pixel(x-1, y, max_tex);
      float uy_up = pixel(x, y-1, max_tex);
      float ly_left = pixel(x-1, y, min_tex);
      float ly_up = pixel(x, y-1, min_tex);
      float d = pixel(x, y, output_tex);

      if(y == 0) ly_up = 1.0;
      if(x == 0) ly_left = 1.0;
      float d1 = cost_function(uy_left, ix, ly_left);
      if(d1 < d) {
        d = d1;
        imageStore(output_tex, ivec2(x, y), vec4(vec3(d1), 1));
        imageStore(max_tex, ivec2(x, y), vec4(vec3(max(uy_left, ix)), 1));
        imageStore(min_tex, ivec2(x, y), vec4(vec3(min(ly_left, ix)), 1));
      }

      float d2 = cost_function(uy_up, ix, ly_up);
      if(d2 < d) {
        d = d2;
        imageStore(output_tex, ivec2(x, y), vec4(vec3(d2), 1));
        imageStore(max_tex, ivec2(x, y), vec4(vec3(max(uy_up, ix)), 1));
        imageStore(min_tex, ivec2(x, y), vec4(vec3(min(ly_up, ix)), 1));
      }
    }
  }  
}

void set_up_shared_memory() {
  const ivec2 dim = imageSize(input_tex);

  for(int x = 0; x < dim.x; x++) {
    imageStore(output_tex, ivec2(x, dim.y - 1), vec4(vec3(0), 1));
  }
  for(int y = 0; y < dim.y; y++) {
    imageStore(output_tex, ivec2(dim.x - 1, y), vec4(vec3(0), 1));
  }
}

void main() {
  const uint mode = config.mode;
  const uint num_iterations = 3;
  set_up_shared_memory();
  if(gl_GlobalInvocationID.x == 0 && gl_GlobalInvocationID.y == 0) {
    for(uint i = 1; i <= num_iterations; i++) {
      uint m = i % 2;
      if(m == 1) {
        raster_scan();
      } else {
        inverse_raster_scan();
      }
    }
  }
}

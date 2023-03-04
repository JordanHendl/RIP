#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable
#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 32 
#define BLOCK_SIZE_Z 1 
#define NUM_COLORS   10
#define BLOCK_THREAD_COUNT BLOCK_SIZE_X*BLOCK_SIZE_Y*BLOCK_SIZE_Z

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in; 

layout(binding = 0, rgba32f) coherent restrict writeonly uniform image2D output_tex;

layout( binding = 1 ) buffer Indices {
  uint indices[] ;
} data;

void write(uint ix, uint iy, vec4 color) {
  imageStore(output_tex, ivec2(ix, iy), color);
}

void main() {
  const ivec2 dim = imageSize(output_tex);
  const uint ix  = gl_GlobalInvocationID.x;
  const uint iy  = gl_GlobalInvocationID.y;
  const uint gid = ix + (iy * uint(dim.x));
  
  vec4 colors[10];
  uint position = gid;
  uint root = data.indices[position];
  
  colors[0] = vec4(0.0, 0.0, 1.0, 1.0);
  colors[1] = vec4(0.0, 1.0, 0.0, 1.0);
  colors[2] = vec4(1.0, 0.0, 0.0, 1.0);
  colors[3] = vec4(0.0, 0.5, 0.5, 1.0);
  colors[4] = vec4(0.5, 0.0, 0.5, 1.0);
  colors[5] = vec4(0.5, 0.5, 0.0, 1.0);
  colors[6] = vec4(0.1, 0.3, 0.3, 1.0);
  colors[7] = vec4(0.3, 0.1, 0.7, 1.0);
  colors[8] = vec4(0.0, 0.0, 0.0, 1.0);
  colors[9] = vec4(1.0, 1.0, 1.0, 1.0);
  
  memoryBarrier();
  barrier();

  while(position != data.indices[root]) {
    position = root;
    root = data.indices[root];
  }
  
  memoryBarrier();
  barrier();

  data.indices[gid] = root;
  
  memoryBarrier();
  barrier();

  write(ix, iy, colors[(root % NUM_COLORS)]);
}
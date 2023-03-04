#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_GOOGLE_include_directive    : enable

#define BLOCK_SIZE_X 32 
#define BLOCK_SIZE_Y 32 
#define BLOCK_SIZE_Z 1 
#define BLOCK_THREAD_COUNT BLOCK_SIZE_X*BLOCK_SIZE_Y*BLOCK_SIZE_Z

layout(local_size_x = BLOCK_SIZE_X, local_size_y = BLOCK_SIZE_Y, local_size_z = BLOCK_SIZE_Z) in; 

layout(binding = 0, rgba32f) coherent restrict readonly uniform image2D input_tex;

layout( binding = 1 ) buffer Indices {
  uint indices[];
} data;

bool compare(float a, float b) {
  const float allowed_amt = 0.05f;
  return abs(a - b) < allowed_amt;
}

uint find(uint id) {
  while(id != data.indices[id]) {
    id = data.indices[id];
  }
  return id;
}

void findAndUnion(uint id1, uint id2)
{
  bool done = false;
  uint p = 0;
  uint q = 0;
  
  while(!done) {
    p = find(id1);
    q = find(id2);
    
    if(p < q) {
      atomicMin(data.indices[q] , p);
    } else if(q < p) {
      atomicMin(data.indices[p] , q);
    } else {
      done = true;
    }
  }
}

float pixel(uint x, uint y) {
  return imageLoad(input_tex, ivec2(x, y)).r;
}

void main() {
  const ivec2 dim = imageSize(input_tex);
  const uint ix = gl_GlobalInvocationID.x;
  const uint iy = gl_GlobalInvocationID.y;
  const uint tx = gl_LocalInvocationID.x;
  const uint ty = gl_LocalInvocationID.y;
  const uint gid = ix + (iy * uint(dim.x));

  if(gl_WorkGroupID.x > 0 && tx == 0 && compare(pixel(ix, iy), pixel(ix - 1, iy))) {
    findAndUnion(gid, gid - 1);
  }
  
  if(gl_WorkGroupID.y > 0 && ty == 0 && compare(pixel(ix, iy), pixel(ix, iy - 1))) {
    findAndUnion(gid, gid - uint(dim.x));
  }
}
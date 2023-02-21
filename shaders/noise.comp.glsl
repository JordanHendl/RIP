#version 450 core 
// Linear Congruence Generator
uint lcg(uint seed) {
  return 1664525u * seed + 1013904223u;
}

// Tausworth Generator
uint taus(uint seed, uint s1, uint s2, uint s3, uint m) {
  uint b = (((seed << s1)^seed)>>s2);
  return (((seed & m) << s3) ^ b);
}

void main() {

}
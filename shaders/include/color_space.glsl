vec3 rgb2ycbcr(vec3 c) {
  int r = int(c.r * 256.0);
  int g = int(c.g * 256.0);
  int b = int(c.b * 256.0);

  int y = int(round(0.299*r + 0.587*g + 0.114*b));
  int cr = int(round(128 + 0.5*r - 0.418688*g - 0.081312*b));
  int cb = int(round(128 + -0.168736*r - 0.331264*g + 0.5*b));
  return vec3(float(y) / 256.0, float(cb) / 256.0, float(cr) / 256.0);
}

vec3 ycbcr2rgb(vec3 c) {
  int Y = int(c.r * 256);
  int Cb = int(c.g * 256);
  int Cr = int(c.b * 256);

  int r = int(Y + 1.40200 * (Cr - 128));
  int g = int(Y - 0.34414 * (Cb - 128) - 0.71414 * (Cr - 128));
  int b = int(Y + 1.77200 * (Cb - 128));
  return vec3(float(r) / 256.0, float(g) / 256.0, float(b) / 256.0);
}

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
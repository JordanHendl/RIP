pub fn to_u32_slice(bytes: &[u8]) -> &[u32] {
  unsafe {
    let s = bytes.as_ptr() as *const u32;
    let u = std::slice::from_raw_parts(s, bytes.len() / std::mem::size_of::<u32>());
    return u;
  }
}
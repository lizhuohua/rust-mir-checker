use crate::alloc;

const LEN: usize = 16;
const SIZE: usize = std::mem::size_of::<f32>() * LEN;

#[no_mangle]
pub unsafe fn matrix4determinant(a: *mut f32) -> f32 {
  let a = std::slice::from_raw_parts(a, LEN);

  let b00 = a[0] * a[5] - a[1] * a[4];
  let b01 = a[0] * a[6] - a[2] * a[4];
  let b02 = a[0] * a[7] - a[3] * a[4];
  let b03 = a[1] * a[6] - a[2] * a[5];
  let b04 = a[1] * a[7] - a[3] * a[5];
  let b05 = a[2] * a[7] - a[3] * a[6];
  let b06 = a[8] * a[13] - a[9] * a[12];
  let b07 = a[8] * a[14] - a[10] * a[12];
  let b08 = a[8] * a[15] - a[11] * a[12];
  let b09 = a[9] * a[14] - a[10] * a[13];
  let b10 = a[9] * a[15] - a[11] * a[13];
  let b11 = a[10] * a[15] - a[11] * a[14];

  b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06
}

#[no_mangle]
pub unsafe fn matrix4invert(a: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);

  let b00 = a[0] * a[5] - a[1] * a[4];
  let b01 = a[0] * a[6] - a[2] * a[4];
  let b02 = a[0] * a[7] - a[3] * a[4];
  let b03 = a[1] * a[6] - a[2] * a[5];
  let b04 = a[1] * a[7] - a[3] * a[5];
  let b05 = a[2] * a[7] - a[3] * a[6];
  let b06 = a[8] * a[13] - a[9] * a[12];
  let b07 = a[8] * a[14] - a[10] * a[12];
  let b08 = a[8] * a[15] - a[11] * a[12];
  let b09 = a[9] * a[14] - a[10] * a[13];
  let b10 = a[9] * a[15] - a[11] * a[13];
  let b11 = a[10] * a[15] - a[11] * a[14];
  
  let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

  if det == 0.0 {
    return std::ptr::null_mut();
  }

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);
  let det = 1f32 / det;

  mat[0] = (a[5] * b11 - a[6] * b10 + a[7] * b09) * det;
  mat[1] = (a[2] * b10 - a[1] * b11 - a[3] * b09) * det;
  mat[2] = (a[13] * b05 - a[14] * b04 + a[15] * b03) * det;
  mat[3] = (a[10] * b04 - a[9] * b05 - a[11] * b03) * det;
  mat[4] = (a[6] * b08 - a[4] * b11 - a[7] * b07) * det;
  mat[5] = (a[0] * b11 - a[2] * b08 + a[3] * b07) * det;
  mat[6] = (a[14] * b02 - a[12] * b05 - a[15] * b01) * det;
  mat[7] = (a[8] * b05 - a[10] * b02 + a[11] * b01) * det;
  mat[8] = (a[4] * b10 - a[5] * b08 + a[7] * b06) * det;
  mat[9] = (a[1] * b08 - a[0] * b10 - a[3] * b06) * det;
  mat[10] = (a[12] * b04 - a[13] * b02 + a[15] * b00) * det;
  mat[11] = (a[9] * b02 - a[8] * b04 - a[11] * b00) * det;
  mat[12] = (a[5] * b07 - a[4] * b09 - a[6] * b06) * det;
  mat[13] = (a[0] * b09 - a[1] * b07 + a[2] * b06) * det;
  mat[14] = (a[13] * b01 - a[12] * b03 - a[14] * b00) * det;
  mat[15] = (a[8] * b03 - a[9] * b01 + a[10] * b00) * det;

  ptr
}

#[no_mangle]
pub unsafe fn matrix4mul(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = b[0] * a[0] + b[1] * a[4] + b[2] * a[8] + b[3] * a[12];
  mat[1] = b[0] * a[1] + b[1] * a[5] + b[2] * a[9] + b[3] * a[13];
  mat[2] = b[0] * a[2] + b[1] * a[6] + b[2] * a[10] + b[3] * a[14];
  mat[3] = b[0] * a[3] + b[1] * a[7] + b[2] * a[11] + b[3] * a[15];
  mat[4] = b[4] * a[0] + b[5] * a[4] + b[6] * a[8] + b[7] * a[12];
  mat[5] = b[4] * a[1] + b[5] * a[5] + b[6] * a[9] + b[7] * a[13];
  mat[6] = b[4] * a[2] + b[5] * a[6] + b[6] * a[10] + b[7] * a[14];
  mat[7] = b[4] * a[3] + b[5] * a[7] + b[6] * a[11] + b[7] * a[15];
  mat[8] = b[8] * a[0] + b[9] * a[4] + b[10] * a[8] + b[11] * a[12];
  mat[9] = b[8] * a[1] + b[9] * a[5] + b[10] * a[9] + b[11] * a[13];
  mat[10] = b[8] * a[2] + b[9] * a[6] + b[10] * a[10] + b[11] * a[14];
  mat[11] = b[8] * a[3] + b[9] * a[7] + b[10] * a[11] + b[11] * a[15];
  mat[12] = b[12] * a[0] + b[13] * a[4] + b[14] * a[8] + b[15] * a[12];
  mat[13] = b[12] * a[1] + b[13] * a[5] + b[14] * a[9] + b[15] * a[13];
  mat[14] = b[12] * a[2] + b[13] * a[6] + b[14] * a[10] + b[15] * a[14];
  mat[15] = b[12] * a[3] + b[13] * a[7] + b[14] * a[11] + b[15] * a[15];

  ptr
}

#[no_mangle]
pub unsafe fn matrix4add(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = a[0] + b[0];
  mat[1] = a[1] + b[1];
  mat[2] = a[2] + b[2];
  mat[3] = a[3] + b[3];
  mat[4] = a[4] + b[4];
  mat[5] = a[5] + b[5];
  mat[6] = a[6] + b[6];
  mat[7] = a[7] + b[7];
  mat[8] = a[8] + b[8];
  mat[9] = a[9] + b[9];
  mat[10] = a[10] + b[10];
  mat[11] = a[11] + b[11];
  mat[12] = a[12] + b[12];
  mat[13] = a[13] + b[13];
  mat[14] = a[14] + b[14];
  mat[15] = a[15] + b[15];

  ptr
}

#[no_mangle]
pub unsafe fn matrix4sub(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = a[0] - b[0];
  mat[1] = a[1] - b[1];
  mat[2] = a[2] - b[2];
  mat[3] = a[3] - b[3];
  mat[4] = a[4] - b[4];
  mat[5] = a[5] - b[5];
  mat[6] = a[6] - b[6];
  mat[7] = a[7] - b[7];
  mat[8] = a[8] - b[8];
  mat[9] = a[9] - b[9];
  mat[10] = a[10] - b[10];
  mat[11] = a[11] - b[11];
  mat[12] = a[12] - b[12];
  mat[13] = a[13] - b[13];
  mat[14] = a[14] - b[14];
  mat[15] = a[15] - b[15];

  ptr
}

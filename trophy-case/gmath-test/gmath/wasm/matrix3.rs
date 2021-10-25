use crate::alloc;

const LEN: usize = 9;
const SIZE: usize = std::mem::size_of::<f32>() * LEN;

#[no_mangle]
pub unsafe fn matrix3determinant(a: *mut f32) -> f32 {
  let a = std::slice::from_raw_parts(a, LEN);

  a[0] * (a[8] * a[4] - a[5] * a[7])
    + a[1] * (-a[8] * a[3] + a[5] * a[6])
    + a[2] * (a[7] * a[3] - a[4] * a[6])
}

#[no_mangle]
pub unsafe fn matrix3invert(a: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);

  let b01 = a[8] * a[4] - a[5] * a[7];
  let b11 = -a[8] * a[3] + a[5] * a[6];
  let b21 = a[7] * a[3] - a[4] * a[6];

  let det = a[0] * b01 + a[1] * b11 + a[2] * b21;

  if det == 0.0 {
    return std::ptr::null_mut();
  }

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);
  let det = 1f32 / det;

  mat[0] = b01 * det;
  mat[1] = (-a[8] * a[1] + a[2] * a[7]) * det;
  mat[2] = (a[5] * a[1] - a[2] * a[4]) * det;
  mat[3] = b11 * det;
  mat[4] = (a[8] * a[0] - a[2] * a[6]) * det;
  mat[5] = (-a[5] * a[0] + a[2] * a[3]) * det;
  mat[6] = b21 * det;
  mat[7] = (-a[7] * a[0] + a[1] * a[6]) * det;
  mat[8] = (a[4] * a[0] - a[1] * a[3]) * det;

  ptr
}

#[no_mangle]
pub unsafe fn matrix3mul(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = b[0] * a[0] + b[1] * a[3] + b[2] * a[6];
  mat[1] = b[0] * a[1] + b[1] * a[4] + b[2] * a[7];
  mat[2] = b[0] * a[2] + b[1] * a[5] + b[2] * a[8];
  mat[3] = b[3] * a[0] + b[4] * a[3] + b[5] * a[6];
  mat[4] = b[3] * a[1] + b[4] * a[4] + b[5] * a[7];
  mat[5] = b[3] * a[2] + b[4] * a[5] + b[5] * a[8];
  mat[6] = b[6] * a[0] + b[7] * a[3] + b[8] * a[6];
  mat[7] = b[6] * a[1] + b[7] * a[4] + b[8] * a[7];
  mat[8] = b[6] * a[2] + b[7] * a[5] + b[8] * a[8];

  ptr
}

#[no_mangle]
pub unsafe fn matrix3add(a: *mut f32, b: *mut f32) -> *mut u8 {
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

  ptr
}

#[no_mangle]
pub unsafe fn matrix3sub(a: *mut f32, b: *mut f32) -> *mut u8 {
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

  ptr
}

use crate::alloc;

const LEN: usize = 4;
const SIZE: usize = std::mem::size_of::<f32>() * LEN;

#[no_mangle]
pub unsafe fn matrix2determinant(a: *mut f32) -> f32 {
  let a = std::slice::from_raw_parts(a, LEN);

  a[0] * a[3] - a[2] * a[1]
}

#[no_mangle]
pub unsafe fn matrix2invert(a: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);

  let det = a[0] * a[3] - a[2] * a[1];

  if det == 0.0 {
    return std::ptr::null_mut();
  }

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);
  let det = 1f32 / det;

  mat[0] = a[3] * det;
  mat[1] = -a[1] * det;
  mat[2] = -a[2] * det;
  mat[3] = a[0] * det;

  ptr
}

#[no_mangle]
pub unsafe fn matrix2mul(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = a[0] * b[0] + a[2] * b[1];
  mat[1] = a[1] * b[0] + a[3] * b[1];
  mat[2] = a[0] * b[2] + a[2] * b[3];
  mat[3] = a[1] * b[2] + a[3] * b[3];

  ptr
}

#[no_mangle]
pub unsafe fn matrix2add(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = a[0] + b[0];
  mat[1] = a[1] + b[1];
  mat[2] = a[2] + b[2];
  mat[3] = a[3] + b[3];

  ptr
}

#[no_mangle]
pub unsafe fn matrix2sub(a: *mut f32, b: *mut f32) -> *mut u8 {
  let a = std::slice::from_raw_parts(a, LEN);
  let b = std::slice::from_raw_parts(b, LEN);

  let ptr = alloc(SIZE);
  let mut mat = Vec::from_raw_parts(ptr as *mut f32, LEN, LEN);

  mat[0] = a[0] - b[0];
  mat[1] = a[1] - b[1];
  mat[2] = a[2] - b[2];
  mat[3] = a[3] - b[3];

  ptr
}

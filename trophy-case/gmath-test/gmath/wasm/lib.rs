pub mod matrix2;
pub mod matrix3;
pub mod matrix4;

#[no_mangle]
pub unsafe fn alloc(size: usize) -> *mut u8 {
  let align = std::mem::align_of::<usize>();
  let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
  std::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
  let align = std::mem::align_of::<usize>();
  let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
  std::alloc::dealloc(ptr, layout);
}

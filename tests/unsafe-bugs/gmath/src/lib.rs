const LEN: usize = 4;
const SIZE: usize = std::mem::size_of::<f32>() * LEN;

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

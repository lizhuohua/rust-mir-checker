use gmath::dealloc;
use gmath::matrix2;

fn main() {
    let mut matrix = [1.0, 1.0, 1.0, 0.0];
    unsafe {
        // `matrix2invert` returns a result that is freed
        let result = matrix2::matrix2invert(&mut matrix[0]);
        // This would cause a use-after-free
        // let mat = std::slice::from_raw_parts(result as *mut f32, 4);
        // dealloc(result, std::mem::size_of::<f32>() * 4);

        // let mat = Vec::from_raw_parts(result as *mut f32, 4, 4);
        // println!("outside: {} {} {} {}", mat[0], mat[1], mat[2], mat[3]);
        let ptr = result as *mut f32;
        *ptr.offset(0) = 1.0;
        *ptr.offset(1) = 2.0;
        *ptr.offset(2) = 3.0;
        *ptr.offset(3) = 4.0;
        *ptr.offset(4) = 5.0;
        println!(
            "{} {} {} {} {}",
            *ptr.offset(0),
            *ptr.offset(1),
            *ptr.offset(2),
            *ptr.offset(3),
            *ptr.offset(4),
        );
        // dealloc(result, std::mem::size_of::<f32>() * 4);
    }
}

// This would enter unreachable code
// https://github.com/Lokathor/bytemuck/issues/52

use bytemuck;

fn main() {
    // Panic: enter unreachable code
    let zst: [u32; 0] = [];
    let _result = bytemuck::bytes_of(&zst);
}

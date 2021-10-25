// This will cause a segmentation fault
// https://gitlab.com/nathanfaucett/rs-executable_memory/-/issues/1

use executable_memory::ExecutableMemory;

fn main() {
    let memory = ExecutableMemory::new(2251799813685248);
    println!("len: {}", memory.len());
    println!("read: {}", memory.as_slice()[5000]);
}

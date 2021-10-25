// This would cause an integer overflow
// https://gitlab.com/nathanfaucett/rs-executable_memory/-/issues/1

use executable_memory::ExecutableMemory;

fn main() {
    let _memory = ExecutableMemory::new(std::usize::MAX);
}

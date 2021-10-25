fn main() {
    let insert_at_index: usize = 5;

    let mut buf = vec![1, 2, 3, 4, 5];
    let max_index: usize = 5;
    // if insert_at_index > max_index {
    //     panic!();
    // }
    buf[insert_at_index] = 100;
}

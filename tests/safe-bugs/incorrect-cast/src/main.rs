fn main() {
    let _a = overflow(-1);
}

fn overflow(time: i64) -> u32 {
    (time % 1000) as u32 * 1000000
}

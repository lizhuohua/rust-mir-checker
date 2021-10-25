// Bug in brotli-rs that may trigger unreachable!()

fn main() {
    let n = 100;

    match n {
        1..=96 | 123..=191 => {
            // do something...
        }
        _ => unreachable!(),
    }
}

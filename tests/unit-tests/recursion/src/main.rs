// Make sure that our analysis for recursive calls can terminate
// Otherwise, there will be a stack overflow

#[allow(unused_variables)]
fn main() {
    let result = factorial(5);
}

fn factorial(n: u32) -> u32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

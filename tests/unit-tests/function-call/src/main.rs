#[macro_use]
extern crate macros;

// // side effect on return value
fn side_effect_return(a: i32) -> i32 {
    2 * a
}

// // side effect on mutable argument
fn side_effect_arg(a: &mut i32) {
    *a = *a + 1;
}

// side effect on heap
fn side_effect_heap(heap: &mut [u32]) {
    heap[0] = 100;
}

fn side_effect_nested(heap: &mut [u32]) {
    heap[1] = side_effect_return(3) as u32; // 6
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
fn main() {
    let mut heap = [1, 2, 3, 4, 5];
    let mut r = 5;
    r = side_effect_return(3);
    // Make sure `r` is now 6
    verify!(r == 6);

    side_effect_arg(&mut r);
    // Make sure `r` is now 7
    verify!(r == 7);

    side_effect_heap(&mut heap);
    // Make sure `heap[0]` is now 100
    verify!(heap[0] == 100);

    side_effect_nested(&mut heap);
    // Make sure `heap[1]` is now 6
    verify!(heap[1] == 6);
}

// Proof-of-concept of several double-free vulnerabilities (CVE-2018-20996, CVE-2019-16880, CVE-2019-16144, CVE-2019-16881)
// Not very similar to those real CVEs but should be enough for illustration purpose

pub struct Foo {
    pub s: Vec<u32>,
}

impl Drop for Foo {
    fn drop(&mut self) {
        println!("Dropping: {:?}", self.s);
    }
}

pub fn fun1() -> Foo {
    let mut src = vec![1, 2, 3, 4, 5, 6];
    let foo = fun2(&mut src);
    foo
}

pub fn fun2(src: &mut Vec<u32>) -> Foo {
    let s = unsafe { Vec::from_raw_parts(src.as_mut_ptr(), src.len(), 32) };
    Foo { s: s }
}

pub fn main() {
    let _foo = fun1();
}

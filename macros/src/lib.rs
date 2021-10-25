#[macro_export]
macro_rules! verify {
    ($condition:expr) => {
        if cfg!(mir_checker) {
            macros::mir_checker_verify($condition)
        }
    };
}

// Dummy function
pub fn mir_checker_verify(_condition: bool) {}

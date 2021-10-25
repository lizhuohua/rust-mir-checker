use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct Dataset {
    /// The number of symmetry operations.
    pub n_operations: i32,
    /// The rotation symmetry operations.
    pub rotations: Vec<[[i32; 3]; 3]>,
    /// The translation symmetry operations.
    pub translations: Vec<[f64; 3]>,
}

// Pretends to be a C style structure
#[derive(Clone, Debug)]
pub struct SpglibDataset {
    /// The number of symmetry operations.
    pub n_operations: i32,
    /// The rotation symmetry operations.
    pub rotations: *mut [[i32; 3]; 3],
    /// The translation symmetry operations.
    pub translations: *mut [f64; 3],
}

impl TryFrom<*mut SpglibDataset> for Dataset {
    type Error = &'static str;

    fn try_from(value: *mut SpglibDataset) -> Result<Self, Self::Error> {
        // dereference the raw pointer
        let ptr = unsafe { &mut *value };
        let n_operations = ptr.n_operations as i32;
        let rotations = unsafe {
            // This creates possible mutable shared memory
            Vec::from_raw_parts(ptr.rotations, n_operations as usize, n_operations as usize)
        };
        let translations = unsafe {
            Vec::from_raw_parts(
                ptr.translations,
                n_operations as usize,
                n_operations as usize,
            )
        };
        Ok(Dataset {
            n_operations,
            rotations,
            translations,
        })
    }
}

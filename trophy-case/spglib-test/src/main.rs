use spglib::dataset::Dataset;
use spglib_sys::spg_get_dataset;
use std::convert::TryFrom;

fn main() {
    let mut lattice = [[4.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 3.0]];
    // This unsafe is OK, it calls an external C API to construct a dataset
    let spglib_dataset_ptr =
        unsafe { spg_get_dataset(&mut lattice[0], &mut [0.0, 0.0, 0.0], &1, 1, 0.00001) };

    // This would cause a double-free, because `try_from` gets ownership from input
    // So we basically construct two aliases of the dataset
    let dataset1 = Dataset::try_from(spglib_dataset_ptr);
    let dataset2 = Dataset::try_from(spglib_dataset_ptr);
}

// This would cause a division-by-zero
// https://github.com/mir-protocol/r1cs/issues/11
use r1cs::Bn128;
use r1cs::Element;
use r1cs::MdsMatrix;
use r1cs::RescueBuilder;

fn main() {
    let mut builder = RescueBuilder::<Bn128>::new(0);
    let matrix = MdsMatrix::<Bn128>::new(vec![vec![Element::zero()]]);
    builder.mds_matrix(matrix);
    builder.build();
}

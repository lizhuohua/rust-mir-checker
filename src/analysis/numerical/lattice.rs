/// Generic API for lattices
pub trait LatticeTrait {
    fn top() -> Self;
    fn is_top(&self) -> bool;
    fn set_to_top(&mut self);
    fn bottom() -> Self;
    fn is_bottom(&self) -> bool;
    fn set_to_bottom(&mut self);
    fn lub(&self, other: &Self) -> Self;
    fn widening_with(&self, other: &Self) -> Self;
}

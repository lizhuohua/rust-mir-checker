use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};

pub trait CheckerTrait<'tcx, 'a, 'b, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn new(body_visitor: &'b mut WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>) -> Self;

    fn run(&mut self);
}

// A thin wrapper for Apron Numerical Abstract Domain Library

use crate::analysis::memory::path::Path;
use crate::analysis::numerical::interval::{Bound, Interval};
use crate::analysis::numerical::lattice::LatticeTrait;
use crate::analysis::numerical::linear_constraint::{
    LinearConstraint, LinearConstraintSystem, LinearExpression,
};
use crate::analysis::option::AbstractDomainType;
use apron_sys;
use foreign_types::foreign_type;
use foreign_types::{ForeignType, ForeignTypeRef, Opaque};
use rug::{Assign, Integer, Rational};
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

/// The operators that numerical abstract domain supports
pub enum ApronOperation {
    // Binop
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    And,
    Or,
    Xor,
    // Unop
    Not,
    Neg,
}

impl ApronOperation {
    /// Check whether the operator is elementary arithmetic
    /// Because Apron seems to only support elementary arithmetic
    fn is_elementary(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Rem
        )
    }
}

/// The following types are used as parameters of `ApronAbstractDomain`,
/// indicating which kind of abstract domain should be used.
/// All of them implement the `ApronDomainType` trait
#[derive(Clone)]
// Apron Interval
pub struct ApronInterval;
#[derive(Clone)]
// Apron Octagon
pub struct ApronOctagon;
#[derive(Clone)]
// Apron NewPolka Convex Polyhedra
pub struct ApronPolyhedra;
#[derive(Clone)]
// Apron NewPolka Linear Equalities
pub struct ApronLinearEqualities;
#[derive(Clone)]
// Apron PPL Convex Polyhedra
pub struct ApronPplPolyhedra;
#[derive(Clone)]
// Apron PPL Linear Congruences
pub struct ApronPplLinearCongruences;
#[derive(Clone)]
// Apron Reduced Product of NewPolka Convex Polyhedra and PPL Linear Congruences
pub struct ApronPkgridPolyhedraLinCongruences;

pub trait ApronDomainType: Clone {}

impl ApronDomainType for ApronInterval {}
impl ApronDomainType for ApronOctagon {}
impl ApronDomainType for ApronPolyhedra {}
impl ApronDomainType for ApronLinearEqualities {}
impl ApronDomainType for ApronPplPolyhedra {}
impl ApronDomainType for ApronPplLinearCongruences {}
impl ApronDomainType for ApronPkgridPolyhedraLinCongruences {}

/// Different types of abstract domains have different managers
/// So we define a trait to share the same function name `get_manager`
/// `get_domain_type` is used to determine the current type of abstract domain
/// This is useful when different domains need to be handled in different strategies
pub trait GetManagerTrait {
    fn get_manager() -> Rc<ApronManager>;
    fn get_domain_type() -> AbstractDomainType;
}

// Define wrappers for `ap_manager_t`
// Used to represent an Apron manager
foreign_type! {
    pub unsafe type ApronManager : Sync + Send
    {
        type CType = apron_sys::ap_manager_t;
        fn drop = apron_sys::ap_manager_free;
    }
}

// Define wrappers for `ap_abstract0_t`
// Used to represent an abstract state
pub struct AbstractStateRef(Opaque);

unsafe impl ForeignTypeRef for AbstractStateRef {
    type CType = apron_sys::ap_abstract0_t;
}

pub struct AbstractState(NonNull<apron_sys::ap_abstract0_t>);

unsafe impl Sync for AbstractStateRef {}
unsafe impl Send for AbstractStateRef {}

unsafe impl Sync for AbstractState {}
unsafe impl Send for AbstractState {}

impl Drop for AbstractState {
    fn drop(&mut self) {
        unsafe {
            apron_sys::ap_abstract0_free(APRON_MANAGER.clone().unwrap().as_ptr(), self.as_ptr())
        }
    }
}

unsafe impl ForeignType for AbstractState {
    type CType = apron_sys::ap_abstract0_t;
    type Ref = AbstractStateRef;

    unsafe fn from_ptr(ptr: *mut apron_sys::ap_abstract0_t) -> AbstractState {
        AbstractState(NonNull::new_unchecked(ptr))
    }

    fn as_ptr(&self) -> *mut apron_sys::ap_abstract0_t {
        self.0.as_ptr()
    }
}

/// Apron library uses a global manager to handle abstract domains
static mut APRON_MANAGER: Option<Rc<ApronManager>> = None;

/// Represent an Apron abstract domain, whose domain type is specified by parameter `Type`
pub struct ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
{
    abstract_state: AbstractState,
    var_map: BTreeMap<Rc<Path>, apron_sys::ap_dim_t>,
    phantom: PhantomData<Type>, // This just makes the compiler happy
}

// Abstract state is stored as a pointer to the C type `ap_abstract0_t`
// So merely cloning the pointer will create two domains that share the same abstract state
// To clone a domain, we need to manually copy the abstract state by using the Apron C API `ap_abstract0_copy`
impl<Type> Clone for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn clone(&self) -> Self {
        Self {
            var_map: self.var_map.clone(),
            phantom: self.phantom,
            abstract_state: unsafe {
                AbstractState::from_ptr(apron_sys::ap_abstract0_copy(
                    Self::get_manager().as_ptr(),
                    self.abstract_state.as_ptr(),
                ))
            },
        }
    }
}

// The following `impl`s implement the `get_manager` function for different kinds of Apron domains
// They simply call their corresponding manager allocation API if the manager is not created yet
impl GetManagerTrait for ApronAbstractDomain<ApronInterval> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(apron_sys::box_manager_alloc()));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Interval
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPolyhedra> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(apron_sys::pk_manager_alloc(false)));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Polyhedra
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronOctagon> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(apron_sys::oct_manager_alloc()));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::Octagon
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronLinearEqualities> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(apron_sys::pkeq_manager_alloc()));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::LinearEqualities
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPplPolyhedra> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(
                    apron_sys::ap_ppl_poly_manager_alloc(false),
                ));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PplPolyhedra
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPplLinearCongruences> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man = Rc::new(ApronManager::from_ptr(
                    apron_sys::ap_ppl_grid_manager_alloc(),
                ));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PplLinearCongruences
    }
}

impl GetManagerTrait for ApronAbstractDomain<ApronPkgridPolyhedraLinCongruences> {
    fn get_manager() -> Rc<ApronManager> {
        if let Some(apron_man) = unsafe { APRON_MANAGER.clone() } {
            apron_man
        } else {
            unsafe {
                let apron_man =
                    Rc::new(ApronManager::from_ptr(apron_sys::ap_pkgrid_manager_alloc(
                        apron_sys::pk_manager_alloc(false),
                        apron_sys::ap_ppl_grid_manager_alloc(),
                    )));
                APRON_MANAGER = Some(apron_man.clone());
                apron_man
            }
        }
    }

    fn get_domain_type() -> AbstractDomainType {
        AbstractDomainType::PkgridPolyhedraLinCongruences
    }
}

impl<Type> Default for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn default() -> Self {
        Self::top()
    }
}

// Abstract domain forms a lattice
impl<Type> LatticeTrait for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn top() -> Self {
        let abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_top(
                Self::get_manager().as_ptr(),
                0,
                0,
            ))
        };

        Self {
            abstract_state,
            var_map: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    fn bottom() -> Self {
        let abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_bottom(
                Self::get_manager().as_ptr(),
                0,
                0,
            ))
        };

        Self {
            abstract_state,
            var_map: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    fn set_to_top(&mut self) {
        let abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_top(
                Self::get_manager().as_ptr(),
                0,
                0,
            ))
        };
        *self = Self {
            abstract_state,
            var_map: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    fn set_to_bottom(&mut self) {
        let abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_bottom(
                Self::get_manager().as_ptr(),
                0,
                0,
            ))
        };
        *self = Self {
            abstract_state,
            var_map: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    fn is_top(&self) -> bool {
        unsafe {
            apron_sys::ap_abstract0_is_top(
                Self::get_manager().as_ptr(),
                self.abstract_state.as_ptr(),
            )
        }
    }

    fn is_bottom(&self) -> bool {
        unsafe {
            apron_sys::ap_abstract0_is_bottom(
                Self::get_manager().as_ptr(),
                self.abstract_state.as_ptr(),
            )
        }
    }

    fn lub(&self, other: &Self) -> Self {
        self.join(other)
    }

    fn widening_with(&self, other: &Self) -> Self {
        self.widening_with(other)
    }
}

impl<Type> ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    // This is to make sure `Self` can be converted, i.e., `<Self as GetManagerTrait>`
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    /// Determine whether `lhs <= rhs`, with respect to the partial ordering defined by lattice
    pub fn leq(&self, other: &Self) -> bool {
        if self.is_bottom() {
            true
        } else if other.is_bottom() {
            false
        } else if other.is_top() {
            true
        } else if self.is_top() && !other.is_top() {
            false
        } else if self.is_top() && other.is_top() {
            true
        } else {
            // Seems like `ap_abstract0_is_leq` will panic for some domains if two operands have different dimensions
            if self.get_dims() != other.get_dims() {
                warn!("When comparing two ApronAbstractDomain, two operands have different dimensions");
                false
            } else {
                let join = self.join(other);
                unsafe {
                    apron_sys::ap_abstract0_is_leq(
                        Self::get_manager().as_ptr(),
                        join.abstract_state.as_ptr(),
                        other.abstract_state.as_ptr(),
                    )
                }
            }
        }
    }

    /// Used to handle move assignments: `new_path = old_path;`
    /// `new_path` get the value of `old_path`, and `old_path` goes out of scope
    /// The overall effect is equivalent to renaming `old_path` to `new_path`
    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if self.contains(old_path) {
            self.assign_var(new_path.clone(), old_path.clone());
            self.forget(old_path);
        }
    }

    /// Used to handle copy assignments: `new_path = old_path;`
    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if self.contains(old_path) {
            self.assign_var(new_path.clone(), old_path.clone());
        }
    }

    /// Get a list of paths that are in the current domain
    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        self.var_map.keys().cloned().collect()
    }

    /// Determine whether `path` is in the current domain
    pub fn contains(&self, path: &Rc<Path>) -> bool {
        self.var_map.contains_key(path)
    }

    /// Get a reference to `AbstractState`
    pub fn get_state(&self) -> &AbstractState {
        &self.abstract_state
    }

    /// Get a reference to `ApronManager`
    pub fn get_manager() -> Rc<ApronManager> {
        <Self as GetManagerTrait>::get_manager()
    }

    /// Get abstract value according to the given path, and transform it to an interval
    pub fn get_interval(&self, var: &Rc<Path>) -> Interval {
        self.var2itv(var)
    }

    /// Handle assignment `var = n` where n is a constant integer
    pub fn assign_int(&mut self, var: Rc<Path>, n: Integer) {
        self.assign_linexpr(var, &LinearExpression::from(n));
    }

    /// Handle assignment `var = rvalue` where `rvalue` is a path
    pub fn assign_var(&mut self, var: Rc<Path>, rvalue: Rc<Path>) {
        self.assign_linexpr(var, &(LinearExpression::default() + rvalue));
    }

    /// Compute narrowing
    pub fn narrowing_with(&self, rhs: &Self) -> Self {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() {
            rhs.clone()
        } else if rhs.is_top() {
            self.clone()
        } else {
            let mut res = self.clone();
            let mut other = rhs.clone();

            let new_var_map = Self::merge_var_map(&mut res, &mut other);
            match Self::get_domain_type() {
                AbstractDomainType::Octagon => {
                    res.var_map = new_var_map;
                    res.abstract_state = unsafe {
                        AbstractState::from_ptr(apron_sys::ap_abstract0_oct_narrowing(
                            Self::get_manager().as_ptr(),
                            res.get_state().as_ptr(),
                            other.get_state().as_ptr(),
                        ))
                    };
                    res
                }
                // FIXME: use meet instead of narrowing.
                // Make sure iterations will terminate
                _ => self.meet(rhs),
            }
        }
    }

    /// Compute widening
    pub fn widening_with(&self, rhs: &Self) -> Self {
        let mut res = self.clone();
        let mut other = rhs.clone();

        let new_var_map = Self::merge_var_map(&mut res, &mut other);
        res.var_map = new_var_map;
        res.abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_widening(
                Self::get_manager().as_ptr(),
                res.get_state().as_ptr(),
                other.get_state().as_ptr(),
            ))
        };
        res
    }

    /// Compute the least upper bound
    pub fn join(&self, rhs: &Self) -> Self {
        if self.is_bottom() || rhs.is_top() {
            rhs.clone()
        } else if self.is_top() || rhs.is_bottom() {
            self.clone()
        } else {
            let mut res = self.clone();
            let mut other = rhs.clone();

            let new_var_map = Self::merge_var_map(&mut res, &mut other);
            res.var_map = new_var_map;
            // debug!("Merged Var Map: {:?}", res.var_map);
            res.abstract_state = unsafe {
                AbstractState::from_ptr(apron_sys::ap_abstract0_join(
                    Self::get_manager().as_ptr(),
                    false,
                    res.get_state().as_ptr(),
                    other.get_state().as_ptr(),
                ))
            };
            res
        }
    }

    /// Compute the greatest lower bound
    pub fn meet(&self, rhs: &Self) -> Self {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() {
            rhs.clone()
        } else if rhs.is_top() {
            self.clone()
        } else {
            let mut res = self.clone();
            let mut other = rhs.clone();

            let new_var_map = Self::merge_var_map(&mut res, &mut other);
            res.var_map = new_var_map;
            res.abstract_state = unsafe {
                AbstractState::from_ptr(apron_sys::ap_abstract0_meet(
                    Self::get_manager().as_ptr(),
                    false,
                    res.get_state().as_ptr(),
                    other.get_state().as_ptr(),
                ))
            };
            res
        }
    }

    /// Apply the binary operation statement: `res = lhs op rhs`
    pub fn apply_bin_op_place_place(
        &mut self,
        op: ApronOperation,
        lhs: &Rc<Path>,
        rhs: &Rc<Path>,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            if op.is_elementary() {
                // Use apron library
                let lhs_expr = self.var2texpr(lhs);
                let rhs_expr = self.var2texpr(rhs);
                self.do_bin_op_expr(op, lhs_expr, rhs_expr, res);
            } else {
                // Use interval operations
                let lhs_itv = self.var2itv(lhs);
                let rhs_itv = self.var2itv(rhs);
                self.do_bin_op_itv(op, lhs_itv, rhs_itv, res);
            }
        }
    }

    /// Apply the binary operation statement: `res = cst op rhs`
    pub fn apply_bin_op_const_place(
        &mut self,
        op: ApronOperation,
        cst: &Integer,
        rhs: &Rc<Path>,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            if op.is_elementary() {
                // Use apron library
                let lhs_expr = Self::num2texpr(cst);
                let rhs_expr = self.var2texpr(rhs);
                self.do_bin_op_expr(op, lhs_expr, rhs_expr, res);
            } else {
                // Use interval operations
                let lhs_itv = Interval::new(Bound::from(cst.clone()), Bound::from(cst.clone()));
                let rhs_itv = self.var2itv(rhs);
                self.do_bin_op_itv(op, lhs_itv, rhs_itv, res);
            }
        }
    }

    /// Apply the binary operation statement: `res = lhs op cst`
    pub fn apply_bin_op_place_const(
        &mut self,
        op: ApronOperation,
        lhs: &Rc<Path>,
        cst: &Integer,
        res: &Rc<Path>,
    ) {
        if !self.is_bottom() {
            if op.is_elementary() {
                // Use apron library
                let lhs_expr = self.var2texpr(lhs);
                let rhs_expr = Self::num2texpr(cst);
                self.do_bin_op_expr(op, lhs_expr, rhs_expr, res);
            } else {
                // Use interval operations
                let lhs_itv = self.var2itv(lhs);
                let rhs_itv = Interval::new(Bound::from(cst.clone()), Bound::from(cst.clone()));
                self.do_bin_op_itv(op, lhs_itv, rhs_itv, res);
            }
        }
    }

    /// Apply the unary operation statement: `res = - rhs`, or `res = !rhs`
    pub fn apply_un_op_place(&mut self, op: ApronOperation, rhs: &Rc<Path>, res: &Rc<Path>) {
        if !self.is_bottom() {
            let rhs_expr = self.var2texpr(rhs);
            let res_expr = match op {
                ApronOperation::Neg => Self::neg(rhs_expr),
                // TODO: implement not operation
                ApronOperation::Not => unreachable!(),
                _ => unreachable!("Undefined UnOp, this is a bug"),
            };
            let dim_res = self.get_var_dim_insert(res.clone());
            unsafe {
                self.abstract_state =
                    AbstractState::from_ptr(apron_sys::ap_abstract0_assign_texpr(
                        Self::get_manager().as_ptr(),
                        false,
                        self.abstract_state.as_ptr(),
                        dim_res,
                        res_expr,
                        std::ptr::null_mut(),
                    ));
                apron_sys::ap_texpr0_free(res_expr);
            }
        }
    }

    /// Remove a path from current abstract domain
    pub fn forget(&mut self, var: &Rc<Path>) {
        let mut vec_dims = Vec::new();
        if let Some(dim) = self.get_var_dim(var) {
            vec_dims.push(dim);
            self.abstract_state = unsafe {
                AbstractState::from_ptr(apron_sys::ap_abstract0_forget_array(
                    Self::get_manager().as_ptr(),
                    false,
                    self.abstract_state.as_ptr(),
                    &mut vec_dims[0] as *mut apron_sys::ap_dim_t,
                    1,
                    false,
                ))
            };

            let mut new_var_map: BTreeMap<Rc<Path>, apron_sys::ap_dim_t> = BTreeMap::new();
            // We have to iterate by the dim to preserve the order
            let mut old_var_map: Vec<(&Rc<Path>, &apron_sys::ap_dim_t)> =
                self.var_map.iter().collect();
            old_var_map.sort_by(|a, b| a.1.cmp(b.1));
            for (var, old_dim) in old_var_map {
                if dim != *old_dim {
                    new_var_map.insert(var.clone(), new_var_map.len() as apron_sys::ap_dim_t);
                }
            }
            self.remove_dimensions(vec_dims);
            self.var_map = new_var_map;
        }
    }

    /// Add a linear constraint system into current abstract domain
    pub fn add_constraints(&mut self, conds: LinearConstraintSystem) {
        if self.is_bottom() {
            return;
        }
        if conds.is_false() {
            self.set_to_bottom();
            return;
        }
        if conds.is_true() {
            return;
        }
        let mut array = unsafe { apron_sys::ap_tcons0_array_make(conds.size()) };

        for (i, cst) in (&conds).into_iter().enumerate() {
            let tcons = self.const2tconst(cst);
            unsafe {
                *array.p.add(i) = tcons;
            }
        }

        self.abstract_state = unsafe {
            AbstractState::from_ptr(apron_sys::ap_abstract0_meet_tcons_array(
                Self::get_manager().as_ptr(),
                false,
                self.abstract_state.as_ptr(),
                &mut array as *mut apron_sys::ap_tcons0_array_t,
            ))
        };

        unsafe {
            apron_sys::ap_tcons0_array_clear(&mut array as *mut apron_sys::ap_tcons0_array_t);
        }
    }

    /// Converting `ap_lincons0_t` into `LinearConstraint`
    // TODO: should we expose such a low-level API?
    pub fn apcons2cons(&self, cons: apron_sys::ap_lincons0_t) -> LinearConstraint {
        unsafe {
            let linexp = cons.linexpr0;
            // For terms
            let mut e = LinearExpression::from(0);
            for i in 0..(*linexp).size {
                let dim;
                let coeff;
                if (*linexp).discr == apron_sys::ap_linexpr_discr_t_AP_LINEXPR_DENSE {
                    dim = i as apron_sys::ap_dim_t;
                    coeff = (*linexp).p.coeff.add(i)
                } else {
                    dim = (*(*linexp).p.linterm.add(i)).dim;
                    coeff = &mut (*(*linexp).p.linterm.add(i)).coeff as *mut apron_sys::ap_coeff_t;
                }

                if apron_sys::ap_coeff_zero(coeff) {
                    continue;
                } else {
                    e.add_term(self.get_variable(dim), Self::coeff2num(coeff));
                }
            }
            // For constant
            let cst = &mut (*linexp).cst as *mut apron_sys::ap_coeff_t;
            if !apron_sys::ap_coeff_zero(cst) {
                e = e + Self::coeff2num(cst);
            }
            // For constraint type
            match cons.constyp {
                apron_sys::ap_constyp_t_AP_CONS_EQ => {
                    // e == k
                    LinearConstraint::Equality(e)
                }
                apron_sys::ap_constyp_t_AP_CONS_SUPEQ => {
                    // e >= k
                    LinearConstraint::LessEq(-e)
                }
                apron_sys::ap_constyp_t_AP_CONS_SUP => {
                    // e > k
                    LinearConstraint::LessThan(-e)
                }
                apron_sys::ap_constyp_t_AP_CONS_EQMOD => LinearConstraint::new_true(),
                apron_sys::ap_constyp_t_AP_CONS_DISEQ => {
                    // e != k
                    LinearConstraint::Inequality(e)
                }
                _ => {
                    unreachable!();
                }
            }
        }
    }

    // The followings are private methods

    fn merge_var_map(lhs: &mut Self, rhs: &mut Self) -> BTreeMap<Rc<Path>, apron_sys::ap_dim_t> {
        // Merge two `var_map`
        assert_eq!(lhs.var_map.len(), lhs.get_dims());
        assert_eq!(rhs.var_map.len(), rhs.get_dims());
        let mut vars: Vec<Rc<Path>> = lhs.var_map.keys().cloned().collect();
        for v in rhs.var_map.keys() {
            if !vars.contains(&v) {
                vars.push(v.clone());
            }
        }
        lhs.add_dimensions(vars.len() - lhs.get_dims());
        rhs.add_dimensions(vars.len() - rhs.get_dims());
        assert_eq!(lhs.get_dims(), rhs.get_dims());

        let mut new_var_map: BTreeMap<Rc<Path>, apron_sys::ap_dim_t> = BTreeMap::new();
        for (i, v) in vars.iter().enumerate() {
            new_var_map.insert(v.clone(), i as apron_sys::ap_dim_t);
        }

        unsafe {
            let perm_x = apron_sys::ap_dimperm_alloc(lhs.get_dims());
            let perm_y = apron_sys::ap_dimperm_alloc(rhs.get_dims());
            let mut xmap1 = vec![0; lhs.get_dims()];
            let mut xmap2 = vec![0; lhs.get_dims()];
            for (var, &old_index) in &lhs.var_map {
                let new_index = new_var_map[var];
                *(*perm_x).dim.offset(old_index as isize) = new_index;
                xmap1[old_index as usize] = 1;
                xmap2[new_index as usize] = 1;
            }
            let mut counter = 0;
            for i in 0..lhs.get_dims() {
                if xmap1[i] == 1 {
                    continue;
                }
                while xmap2[counter] == 1 {
                    counter += 1;
                }
                *(*perm_x).dim.add(i) = counter as u32;
                counter += 1;
            }

            let mut ymap1 = vec![0; lhs.get_dims()];
            let mut ymap2 = vec![0; lhs.get_dims()];
            for (var, &old_index) in &rhs.var_map {
                let new_index = new_var_map[var];
                *(*perm_y).dim.offset(old_index as isize) = new_index;
                ymap1[old_index as usize] = 1;
                ymap2[new_index as usize] = 1;
            }
            let mut counter = 0;
            for i in 0..lhs.get_dims() {
                if ymap1[i] == 1 {
                    continue;
                }
                while ymap2[counter] == 1 {
                    counter += 1;
                }
                *(*perm_y).dim.add(i) = counter as u32;
                counter += 1;
            }

            lhs.abstract_state =
                AbstractState::from_ptr(apron_sys::ap_abstract0_permute_dimensions(
                    Self::get_manager().as_ptr(),
                    false,
                    lhs.abstract_state.as_ptr(),
                    perm_x,
                ));
            rhs.abstract_state =
                AbstractState::from_ptr(apron_sys::ap_abstract0_permute_dimensions(
                    Self::get_manager().as_ptr(),
                    false,
                    rhs.abstract_state.as_ptr(),
                    perm_y,
                ));

            apron_sys::ap_dimperm_free_wrapper(perm_x);
            apron_sys::ap_dimperm_free_wrapper(perm_y);
        }
        new_var_map
    }

    fn assign_linexpr(&mut self, var: Rc<Path>, exp: &LinearExpression) {
        if !self.is_bottom() {
            let texpr = self.expr2texpr(exp);
            let dim = self.get_var_dim_insert(var);
            unsafe {
                self.abstract_state =
                    AbstractState::from_ptr(apron_sys::ap_abstract0_assign_texpr(
                        Self::get_manager().as_ptr(),
                        false,
                        self.abstract_state.as_ptr(),
                        dim,
                        texpr,
                        std::ptr::null_mut(),
                    ));
                apron_sys::ap_texpr0_free(texpr);
            }
        }
    }

    fn get_var_dim(&self, v: &Rc<Path>) -> Option<apron_sys::ap_dim_t> {
        self.var_map.get(v).copied()
    }

    fn get_var_dim_insert(&mut self, var: Rc<Path>) -> apron_sys::ap_dim_t {
        assert_eq!(self.var_map.len(), self.get_dims());
        if let Some(dim) = self.get_var_dim(&var) {
            dim
        } else {
            let dim = self.var_map.len() as apron_sys::ap_dim_t;
            self.var_map.insert(var.clone(), dim);
            self.add_dimensions(1);
            assert_eq!(self.var_map.len(), self.get_dims());
            dim
        }
    }

    fn add_dimensions(&mut self, dims: usize) {
        if dims > 0 {
            unsafe {
                let dim_change = apron_sys::ap_dimchange_alloc(dims, 0);
                for i in 0..dims {
                    (*(*dim_change).dim.add(i)) = self.get_dims() as u32;
                }
                self.abstract_state =
                    AbstractState::from_ptr(apron_sys::ap_abstract0_add_dimensions(
                        Self::get_manager().as_ptr(),
                        false,
                        self.abstract_state.as_ptr(),
                        dim_change,
                        false,
                    ));
                apron_sys::ap_dimchange_free_wrapper(dim_change);
            }
        }
    }

    fn get_dims(&self) -> usize {
        let dims = unsafe {
            apron_sys::ap_abstract0_dimension(
                Self::get_manager().as_ptr(),
                self.abstract_state.as_ptr(),
            )
        };
        dims.intdim
    }

    fn remove_dimensions(&mut self, dims: Vec<apron_sys::ap_dim_t>) {
        if !dims.is_empty() {
            let mut dims = dims;
            dims.sort_unstable();
            unsafe {
                let dim_change = apron_sys::ap_dimchange_alloc(dims.len(), 0);
                for (i, item) in dims.iter().enumerate() {
                    (*(*dim_change).dim.add(i)) = *item;
                }
                self.abstract_state =
                    AbstractState::from_ptr(apron_sys::ap_abstract0_remove_dimensions(
                        Self::get_manager().as_ptr(),
                        false,
                        self.abstract_state.as_ptr(),
                        dim_change,
                    ));
                apron_sys::ap_dimchange_free_wrapper(dim_change);
            }
        }
    }

    fn get_variable(&self, i: apron_sys::ap_dim_t) -> Rc<Path> {
        for (k, v) in &self.var_map {
            if *v == i {
                return k.clone();
            }
        }
        panic!("Demension {} is not used! var_map: {:?}", i, self.var_map);
    }

    fn do_bin_op_expr(
        &mut self,
        op: ApronOperation,
        lhs_expr: *mut apron_sys::ap_texpr0_t,
        rhs_expr: *mut apron_sys::ap_texpr0_t,
        res: &Rc<Path>,
    ) {
        let res_expr = match op {
            ApronOperation::Add => Self::add(lhs_expr, rhs_expr),
            ApronOperation::Sub => Self::sub(lhs_expr, rhs_expr),
            ApronOperation::Mul => Self::mul(lhs_expr, rhs_expr),
            ApronOperation::Div => Self::div(lhs_expr, rhs_expr),
            ApronOperation::Rem => Self::rem(lhs_expr, rhs_expr),
            _ => unreachable!(),
        };
        let dim_res = self.get_var_dim_insert(res.clone());
        unsafe {
            self.abstract_state = AbstractState::from_ptr(apron_sys::ap_abstract0_assign_texpr(
                Self::get_manager().as_ptr(),
                false,
                self.abstract_state.as_ptr(),
                dim_res,
                res_expr,
                std::ptr::null_mut(),
            ));
            apron_sys::ap_texpr0_free(res_expr);
        }
    }

    fn do_bin_op_itv(
        &mut self,
        op: ApronOperation,
        lhs_itv: Interval,
        rhs_itv: Interval,
        res: &Rc<Path>,
    ) {
        let res_itv = match op {
            ApronOperation::And => lhs_itv & rhs_itv,
            ApronOperation::Or => lhs_itv | rhs_itv,
            ApronOperation::Xor => lhs_itv ^ rhs_itv,
            ApronOperation::Shl => lhs_itv << rhs_itv,
            ApronOperation::Shr => lhs_itv >> rhs_itv,
            _ => unreachable!(),
        };
        self.set_interval(res, res_itv);
    }

    fn const2tconst(&mut self, cst: &LinearConstraint) -> apron_sys::ap_tcons0_t {
        match cst {
            LinearConstraint::Equality(expr) => unsafe {
                apron_sys::ap_tcons0_make_wrapper(
                    apron_sys::ap_constyp_t_AP_CONS_EQ,
                    self.expr2texpr(expr),
                    std::ptr::null_mut(),
                )
            },
            LinearConstraint::Inequality(expr) => unsafe {
                apron_sys::ap_tcons0_make_wrapper(
                    apron_sys::ap_constyp_t_AP_CONS_DISEQ,
                    self.expr2texpr(expr),
                    std::ptr::null_mut(),
                )
            },
            LinearConstraint::LessEq(expr) => unsafe {
                apron_sys::ap_tcons0_make_wrapper(
                    apron_sys::ap_constyp_t_AP_CONS_SUPEQ,
                    self.expr2texpr(&-expr.clone()),
                    std::ptr::null_mut(),
                )
            },
            LinearConstraint::LessThan(expr) => unsafe {
                apron_sys::ap_tcons0_make_wrapper(
                    apron_sys::ap_constyp_t_AP_CONS_SUP,
                    self.expr2texpr(&-expr.clone()),
                    std::ptr::null_mut(),
                )
            },
        }
    }

    fn var2itv(&self, var: &Rc<Path>) -> Interval {
        if self.is_bottom() {
            Interval::bottom()
        } else if let Some(dim) = self.get_var_dim(var) {
            unsafe {
                let itv = apron_sys::ap_abstract0_bound_dimension(
                    Self::get_manager().as_ptr(),
                    self.abstract_state.as_ptr(),
                    dim,
                );
                if apron_sys::ap_interval_is_top(itv) {
                    apron_sys::ap_interval_free(itv);
                    Interval::bottom()
                } else {
                    let lb = (*itv).inf;
                    let ub = (*itv).sup;
                    let res = if apron_sys::ap_scalar_infty(lb) == -1 {
                        // [-∞, k]
                        let sup = Self::mpqptr2num((*ub).val.mpq);
                        // apron_sys::ap_interval_free(itv);
                        Interval::new(Bound::NINF, Bound::Int(sup))
                    } else if apron_sys::ap_scalar_infty(ub) == 1 {
                        // [k, ∞]
                        let inf = Self::mpqptr2num((*lb).val.mpq);
                        // apron_sys::ap_interval_free(itv);
                        Interval::new(Bound::Int(inf), Bound::INF)
                    } else {
                        let inf = Self::mpqptr2num((*lb).val.mpq);
                        let sup = Self::mpqptr2num((*ub).val.mpq);
                        // apron_sys::ap_interval_free(itv);
                        Interval::new(Bound::Int(inf), Bound::Int(sup))
                    };
                    apron_sys::ap_interval_free(itv);
                    res
                }
            }
        } else {
            // For unknown variable, return top
            Interval::top()
        }
    }

    fn expr2texpr(&mut self, expr: &LinearExpression) -> *mut apron_sys::ap_texpr0_t {
        let cst = expr.constant();
        let mut res = Self::num2texpr(&cst);
        for (v, n) in expr {
            let term = Self::mul(Self::num2texpr(n), self.var2texpr(v));
            res = Self::add(res, term);
        }
        res
    }

    fn var2texpr(&mut self, var: &Rc<Path>) -> *mut apron_sys::ap_texpr0_t {
        unsafe { apron_sys::ap_texpr0_dim(self.get_var_dim_insert(var.clone())) }
    }

    fn num2texpr(num: &Integer) -> *mut apron_sys::ap_texpr0_t {
        use gmp_mpfr_sys::gmp;
        use std::mem::MaybeUninit;
        let mut mpq_uninit = MaybeUninit::uninit();
        unsafe {
            gmp::mpq_init(mpq_uninit.as_mut_ptr());
            let mut mpq = mpq_uninit.assume_init();
            gmp::mpq_set_z(&mut mpq, num.as_raw());

            let v = apron_sys::ap_texpr0_cst_scalar_mpq(&mpq);
            gmp::mpq_clear(&mut mpq);
            v
        }
    }

    fn coeff2num(coeff: *mut apron_sys::ap_coeff_t) -> Integer {
        let mpq = unsafe { (*(*coeff).val.scalar).val.mpq };
        Self::mpqptr2num(mpq)
    }

    fn mpqptr2num(mpq: apron_sys::mpq_ptr) -> Integer {
        let mpq_t = unsafe { *(mpq as *mut gmp_mpfr_sys::gmp::mpq_t) };
        let rational = unsafe { Rational::from_raw(mpq_t) };
        let mut res = Integer::new();
        res.assign(rational.floor_ref());

        // This is to prevent `mpq_t` from being freed when `rational` goes out of scope
        // Because this `mpq_t` is like a pointer, and another copy of it is maintained inside `ap_coeff_t`,
        // which will be freed in `ap_lincons0_array_clear`, so omitting the next line will cause a double free
        // See https://docs.rs/rug/1.11.0/rug/struct.Rational.html#method.into_raw
        // And https://docs.rs/rug/1.11.0/rug/struct.Rational.html#method.from_raw
        let _mpq_t2 = rational.into_raw();

        res
    }

    fn add(
        a: *mut apron_sys::ap_texpr0_t,
        b: *mut apron_sys::ap_texpr0_t,
    ) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_binop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_ADD,
                a,
                b,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn sub(
        a: *mut apron_sys::ap_texpr0_t,
        b: *mut apron_sys::ap_texpr0_t,
    ) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_binop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_SUB,
                a,
                b,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn mul(
        a: *mut apron_sys::ap_texpr0_t,
        b: *mut apron_sys::ap_texpr0_t,
    ) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_binop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_MUL,
                a,
                b,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn div(
        a: *mut apron_sys::ap_texpr0_t,
        b: *mut apron_sys::ap_texpr0_t,
    ) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_binop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_DIV,
                a,
                b,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn rem(
        a: *mut apron_sys::ap_texpr0_t,
        b: *mut apron_sys::ap_texpr0_t,
    ) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_binop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_MOD,
                a,
                b,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn neg(a: *mut apron_sys::ap_texpr0_t) -> *mut apron_sys::ap_texpr0_t {
        unsafe {
            apron_sys::ap_texpr0_unop(
                apron_sys::ap_texpr_op_t_AP_TEXPR_NEG,
                a,
                apron_sys::ap_texpr_rtype_t_AP_RTYPE_INT,
                apron_sys::ap_texpr_rdir_t_AP_RDIR_NEAREST,
            )
        }
    }

    fn set_interval(&mut self, v: &Rc<Path>, itv: Interval) {
        // Remove variable from abstract domain
        self.forget(v);

        let mut csts = LinearConstraintSystem::default();
        let low = itv.low;
        if low.is_finite() {
            if let Bound::Int(l) = low {
                let mut expr = LinearExpression::from(l);
                expr = expr - v.clone();
                let cst = LinearConstraint::LessEq(expr);
                csts.add(cst);
            }
        }

        let high = itv.high;
        if high.is_finite() {
            if let Bound::Int(h) = high {
                let mut expr = LinearExpression::from(-h);
                expr = expr + v.clone();
                let cst = LinearConstraint::LessEq(expr);
                csts.add(cst);
            }
        }

        if csts.size() > 0 {
            self.add_constraints(csts);
        }
    }
}

impl<Type> Debug for ApronAbstractDomain<Type>
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        if self.is_bottom() {
            res.push_str("⊥");
        } else if self.is_top() {
            res.push_str("⊤");
        } else {
            let constraint_system = LinearConstraintSystem::from(self);
            res.push_str(format!("{:?}", constraint_system).as_str());
        }
        write!(f, "{}", res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apron_domain() {
        let mut domain1 = ApronAbstractDomain::<ApronInterval>::default();
        assert_eq!(domain1.is_top(), true);
        assert_eq!(domain1.is_bottom(), false);
        domain1.set_to_bottom();
        assert_eq!(domain1.is_bottom(), true);
        assert_eq!(domain1.get_dims(), 0);
        domain1.add_dimensions(1);
        assert_eq!(domain1.get_dims(), 1);
        domain1.add_dimensions(5);
        assert_eq!(domain1.get_dims(), 6);
        domain1.remove_dimensions(vec![3, 4, 1, 5]);
        assert_eq!(domain1.get_dims(), 2);
    }

    #[test]
    fn test_apron_interval_domain() {
        let mut inv1 = ApronAbstractDomain::<ApronInterval>::default();
        let mut inv2 = ApronAbstractDomain::<ApronInterval>::default();

        let local_x = Path::new_local(1, 0);
        let local_y = Path::new_local(2, 0);
        let local_z = Path::new_local(3, 0);
        inv1.assign_linexpr(local_x.clone(), &LinearExpression::from(5));
        inv1.assign_linexpr(local_x.clone(), &LinearExpression::from(6));
        inv1.assign_linexpr(local_y.clone(), &LinearExpression::from(10));
        println!("inv1: {:?}", inv1);

        inv2.assign_linexpr(local_x.clone(), &LinearExpression::from(10));
        inv2.assign_linexpr(local_y.clone(), &LinearExpression::from(20));
        println!("inv2: {:?}", inv2);

        let mut inv3 = inv1.join(&inv2);
        println!("inv1 | inv2: {:?}", inv3);
        inv3.apply_bin_op_place_place(ApronOperation::Add, &local_x, &local_y, &local_z);

        println!("z=x+y: {:?}", inv3);
    }

    #[test]
    fn test_apron_octagon_domain() {
        let mut inv1 = ApronAbstractDomain::<ApronOctagon>::default();
        let mut inv2 = ApronAbstractDomain::<ApronOctagon>::default();

        let local_x = Path::new_local(1, 0);
        let local_y = Path::new_local(2, 0);
        let local_z = Path::new_local(3, 0);
        inv1.assign_linexpr(local_x.clone(), &LinearExpression::from(5));
        inv1.assign_linexpr(local_y.clone(), &LinearExpression::from(10));
        println!("inv1: {:?}", inv1);

        inv2.assign_linexpr(local_x.clone(), &LinearExpression::from(10));
        inv2.assign_linexpr(local_y.clone(), &LinearExpression::from(20));
        println!("inv2: {:?}", inv2);

        let mut inv3 = inv1.join(&inv2);
        println!("inv1 | inv2: {:?}", inv3);
        inv3.apply_bin_op_place_place(ApronOperation::Add, &local_x, &local_y, &local_z);

        println!("inv1: {:?}", inv3);
    }

    #[test]
    fn test_apron_polyhedra_domain() {
        let mut inv1 = ApronAbstractDomain::<ApronPolyhedra>::default();
        let mut inv2 = ApronAbstractDomain::<ApronPolyhedra>::default();

        let local_x = Path::new_local(1, 0);
        let local_y = Path::new_local(2, 0);
        let local_z = Path::new_local(3, 0);
        inv1.assign_linexpr(local_x.clone(), &LinearExpression::from(5));
        inv1.assign_linexpr(local_y.clone(), &LinearExpression::from(10));
        println!("inv1: {:?}", inv1);

        inv2.assign_linexpr(local_x.clone(), &LinearExpression::from(10));
        inv2.assign_linexpr(local_y.clone(), &LinearExpression::from(20));
        println!("inv2: {:?}", inv2);

        let mut inv3 = inv1.join(&inv2);
        println!("inv1 | inv2: {:?}", inv3);
        inv3.apply_bin_op_place_place(ApronOperation::Add, &local_x, &local_y, &local_z);

        println!("z=x+y: {:?}", inv3);
    }

    #[test]
    fn test_apron_interval_domain_comparison() {
        let mut inv1 = ApronAbstractDomain::<ApronInterval>::default();
        let mut inv2 = ApronAbstractDomain::<ApronInterval>::default();
        let mut inv3 = ApronAbstractDomain::<ApronInterval>::default();

        let local_x = Path::new_local(1, 0);
        let local_y = Path::new_local(2, 0);
        inv1.assign_linexpr(local_x.clone(), &LinearExpression::from(5));
        inv1.assign_linexpr(local_y.clone(), &LinearExpression::from(10));
        println!("inv1: {:?}", inv1);

        inv2.assign_linexpr(local_x.clone(), &LinearExpression::from(5));
        inv2.assign_linexpr(local_y.clone(), &LinearExpression::from(10));
        println!("inv2: {:?}", inv2);

        inv3.assign_linexpr(local_x.clone(), &LinearExpression::from(10));
        inv3.assign_linexpr(local_y.clone(), &LinearExpression::from(20));
        println!("inv3: {:?}", inv3);

        let inv4 = inv1.join(&inv3);
        println!("inv4 = inv1 | inv3: {:?}", inv4);

        assert!(inv1.leq(&inv4));
    }
}

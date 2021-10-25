use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::diagnostics::DiagnosticCause;
use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::path::{Path, PathEnum, PathSelector};
use crate::analysis::memory::symbolic_value::SymbolicValue;
use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::linear_constraint::{
    LinearConstraint, LinearConstraintSystem, LinearExpression,
};
use crate::analysis::z3_solver::SmtResult;
use crate::analysis::z3_solver::Z3Solver;
use crate::checker::checker_trait::CheckerTrait;
use log::debug;
use rug::Integer;
use rustc_middle::mir;
use rustc_middle::mir::{Terminator, TerminatorKind};
use rustc_middle::ty::Ty;
use std::convert::From;
use std::rc::Rc;

pub struct AssertionChecker<'tcx, 'a, 'b, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    body_visitor: &'b mut WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>,
}

impl<'tcx, 'a, 'b, 'compiler, DomainType> CheckerTrait<'tcx, 'a, 'b, 'compiler, DomainType>
    for AssertionChecker<'tcx, 'a, 'b, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn new(body_visitor: &'b mut WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>) -> Self {
        Self { body_visitor }
    }

    fn run(&mut self) {
        info!("====== Assertion Checker starts ======");
        let basic_blocks = self.body_visitor.wto.basic_blocks().clone();
        for (bb, bb_data) in basic_blocks.iter_enumerated() {
            let term = bb_data.terminator();
            let post = self.body_visitor.post.clone();
            if let Some(s) = post.get(&bb) {
                self.run_terminator(term, s);
            }
        }
        info!("====== Assertion Checker ends ======");
    }
}

pub enum CheckerResult {
    Safe,    // Proved to be safe
    Unsafe,  // Proved to be unsafe
    Warning, // Do not know whether safe or not
}

impl<'tcx, 'a, 'b, 'compiler, DomainType> AssertionChecker<'tcx, 'a, 'b, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn run_terminator(
        &mut self,
        term: &Terminator<'tcx>,
        abstract_value: &AbstractDomain<DomainType>,
    ) {
        let Terminator { source_info, kind } = term;
        let span = source_info.span;
        if let TerminatorKind::Assert {
            cond,
            expected,
            msg,
            ..
        } = &kind
        {
            debug!(
                "Checking assertion: {:?} with message: {:?}, exptected: {}",
                term, msg, expected
            );
            debug!("Current state: {:?}", abstract_value);

            if let Some(place) = cond.place() {
                if let Some(cond_val) = self.body_visitor.place_to_abstract_value.get(&place) {
                    debug!("place: {:?}, cond_val: {:?}", place, cond_val);
                    let cond_val = cond_val.clone();
                    let check_result = match msg {
                        mir::AssertKind::Overflow(..) => {
                            self.check_overflow(cond_val.clone(), *expected, abstract_value)
                        }
                        _ => self.check_assert_condition(cond_val, *expected, abstract_value),
                    };

                    match check_result {
                        CheckerResult::Safe => (),
                        CheckerResult::Unsafe => {
                            let error = self.body_visitor.context.session.struct_span_warn(
                                span,
                                format!(
                                    "[MirChecker] Provably error: {:?}",
                                    self.body_visitor.recover_var_name(msg)
                                )
                                .as_str(),
                            );
                            self.body_visitor.emit_diagnostic(
                                error,
                                false,
                                DiagnosticCause::from(msg),
                            );
                        }
                        CheckerResult::Warning => {
                            let warning = self.body_visitor.context.session.struct_span_warn(
                                span,
                                format!(
                                    "[MirChecker] Possible error: {:?}",
                                    self.body_visitor.recover_var_name(msg)
                                )
                                .as_str(),
                            );
                            self.body_visitor.emit_diagnostic(
                                warning,
                                false,
                                DiagnosticCause::from(msg),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn check_assert_condition(
        &self,
        cond: Rc<SymbolicValue>,
        expect: bool,
        abstract_value: &AbstractDomain<DomainType>,
    ) -> CheckerResult {
        let solver = &self.body_visitor.z3_solver;

        Self::add_numerical_constraints(&solver, abstract_value);

        let result;
        debug!("In converting assertion condition: {:?}", cond);
        let z3_cond_expr = if expect == false {
            solver.make_not_z3_expression(
                solver.convert_to_bool_sort(solver.get_symbolic_as_z3_expression(&cond)),
            )
        } else {
            solver.convert_to_bool_sort(solver.get_symbolic_as_z3_expression(&cond))
        };
        match solver.solve_expression(&z3_cond_expr) {
            SmtResult::Unsat => {
                // assert is always false
                result = CheckerResult::Unsafe;
            }
            SmtResult::Sat => {
                // assert is satisfiable, now check whether `not cond_val` is always false
                let cst = solver.make_not_z3_expression(z3_cond_expr);
                if solver.solve_expression(&cst) == SmtResult::Unsat {
                    // `not cond_val` is always false, so `cond_val` is always true
                    result = CheckerResult::Safe;
                } else {
                    result = CheckerResult::Warning;
                }
            }
            SmtResult::Unknown => {
                result = CheckerResult::Warning;
            }
        }
        solver.reset();

        result
    }

    // Add constraints from the numerical abstract domain
    fn add_numerical_constraints(solver: &Z3Solver, abstract_value: &AbstractDomain<DomainType>) {
        let constraint_system = LinearConstraintSystem::from(&abstract_value.numerical_domain);

        for cst in &constraint_system {
            solver.assert(&solver.get_as_z3_expression(cst));
        }
    }

    pub fn check_within_range(
        &self,
        path: Rc<Path>,
        val_ty: Ty<'tcx>,
        abstract_value: &AbstractDomain<DomainType>,
    ) -> CheckerResult {
        let solver = &self.body_visitor.z3_solver;
        Self::add_numerical_constraints(&solver, abstract_value);

        let exp_type: ExpressionType = val_ty.kind().into();

        // First solve `var <= max` and `var >= min`
        let mut exp = LinearExpression::default();
        exp = exp - exp_type.max_value_int() + path.clone();
        let cst1 = LinearConstraint::LessEq(exp);

        let mut exp = LinearExpression::default();
        exp = exp + exp_type.min_value_int() - path;
        let cst2 = LinearConstraint::LessEq(exp);

        let within_range_cond = solver.make_and_z3_expression(&cst1, &cst2);

        let result;
        match solver.solve_expression(&within_range_cond) {
            SmtResult::Unsat => {
                // value is not within the valid range
                result = CheckerResult::Unsafe;
            }
            SmtResult::Sat => {
                // value may be within the valid range, now check whether `not within the range` is always false
                if solver.solve_expression(&solver.make_not_z3_expression(within_range_cond))
                    == SmtResult::Unsat
                {
                    // `not within the range` is always false, so `value is within the range` is always true
                    result = CheckerResult::Safe;
                } else {
                    result = CheckerResult::Warning;
                }
            }
            SmtResult::Unknown => {
                result = CheckerResult::Warning;
            }
        }
        solver.reset();
        result
    }

    fn check_overflow(
        &mut self,
        value: Rc<SymbolicValue>,
        expected: bool,
        abstract_value: &AbstractDomain<DomainType>,
    ) -> CheckerResult {
        match &value.expression {
            Expression::CompileTimeConstant(ConstantValue::Int(constant)) => {
                if constant == &Integer::from(expected) {
                    CheckerResult::Safe
                } else {
                    CheckerResult::Unsafe
                }
            }
            Expression::Variable { path, var_type } => {
                assert_eq!(*var_type, ExpressionType::Bool);
                match &path.value {
                    // Overflow operand is a qualified path, meaning that it is the compiler generated bit indicating overflow: `path.1`
                    // The value is in `path.0`, so we construct it manually and test whether its value is possibly an overflow
                    PathEnum::QualifiedPath {
                        length: _,
                        qualifier,
                        selector,
                    } => {
                        if Rc::new(PathSelector::Field(1)) != *selector {
                            unreachable!("selector is not field 1");
                        } else {
                            let new_path = Path::new_field(qualifier.clone(), 0);
                            if let Some(rustc_type) =
                                self.body_visitor.type_visitor.path_ty_cache.get(&new_path)
                            {
                                self.check_within_range(new_path, rustc_type, abstract_value)
                            } else {
                                unreachable!(
                                    "Value that we want to test does not have type infomation"
                                );
                            }
                        }
                    }
                    _ => {
                        // Overflow operand is not a qualified path, meaning that we can test it directly
                        self.check_assert_condition(value, expected, abstract_value)
                    }
                }
            }
            _ => {
                unreachable!("Overflow operand is not a path");
            }
        }
    }
}

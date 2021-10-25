// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::diagnostics::DiagnosticCause;
use crate::analysis::memory::constant_value::{ConstantValue, FunctionReference};
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::known_names::KnownNames;
use crate::analysis::memory::path::{Path, PathRefinement};
use crate::analysis::memory::symbolic_value::{self, SymbolicValue, SymbolicValueTrait};
use crate::analysis::mir_visitor::block_visitor::BlockVisitor;
use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::checker::assertion_checker::{AssertionChecker, CheckerResult};
use crate::checker::checker_trait::CheckerTrait;
use itertools::Itertools;
use rustc_hir::def_id::DefId;
use rustc_middle::mir;
use rustc_middle::ty::subst::SubstsRef;
use rustc_middle::ty::{Ty, TyKind};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub struct CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// The upper layer block visitor
    pub block_visitor: &'call mut BlockVisitor<'tcx, 'analysis, 'block, 'compilation, DomainType>,

    /// The callee's DefId
    pub callee_def_id: DefId,

    /// The callee's FunctionReference
    pub callee_func_ref: Option<Rc<FunctionReference>>,

    /// The callee's SymbolicValue
    pub callee_fun_val: Rc<SymbolicValue>,

    /// The callee's generic argument list
    pub callee_generic_arguments: Option<SubstsRef<'tcx>>,

    /// The callee's KnownNames
    pub callee_known_name: KnownNames,

    /// The callee's generic arguments' types
    pub callee_generic_argument_map: Option<HashMap<rustc_span::Symbol, Ty<'tcx>>>,

    pub args: &'call [mir::Operand<'tcx>],

    /// The actual arguments of the callee, the paths and symbolic values are from the caller
    pub actual_args: &'call [(Rc<Path>, Rc<SymbolicValue>)],

    /// The list of types of the actual arguments
    pub actual_argument_types: &'call [Ty<'tcx>],

    /// The destination where the return value is assigned
    pub destination: Option<(mir::Place<'tcx>, mir::BasicBlock)>,

    /// If the arguments are functions, store them
    pub function_constant_args: &'call [(Rc<Path>, Rc<SymbolicValue>)],

    /// The call stack, used to detect recursive calls
    pub call_stack: Vec<DefId>,
}

impl<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType> Debug
    for CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        "CallVisitor".fmt(f)
    }
}

impl<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
    CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    pub(crate) fn new(
        block_visitor: &'call mut BlockVisitor<'tcx, 'analysis, 'block, 'compilation, DomainType>,
        callee_def_id: DefId,
        callee_generic_arguments: Option<SubstsRef<'tcx>>,
        callee_generic_argument_map: Option<HashMap<rustc_span::Symbol, Ty<'tcx>>>,
        func_const: ConstantValue,
    ) -> CallVisitor<'call, 'block, 'analysis, 'compilation, 'tcx, DomainType> {
        if let ConstantValue::Function(func_ref) = &func_const {
            let callee_known_name = func_ref.known_name;
            let active_calls = block_visitor.body_visitor.call_stack.clone();
            CallVisitor {
                block_visitor, // This is a reference to the caller's block visitor
                callee_def_id,
                callee_func_ref: Some(func_ref.clone()),
                callee_fun_val: Rc::new(func_const.into()),
                callee_generic_arguments,
                callee_known_name,
                callee_generic_argument_map,
                args: &[],
                actual_args: &[],
                actual_argument_types: &[],
                destination: None,
                function_constant_args: &[],
                call_stack: active_calls,
            }
        } else {
            unreachable!("caller should supply a constant function")
        }
    }

    /// Analyze the function based on the current environment (caller's state) and return the post state
    pub fn create_function_post_state(&mut self) -> AbstractDomain<DomainType> {
        debug!(
            "Creating callee's post state, def_id={:?}, type of def_id={:?}",
            self.callee_def_id,
            self.block_visitor
                .body_visitor
                .context
                .tcx
                .type_of(self.callee_def_id)
        );
        // If MIR is available, analyze it
        if self
            .block_visitor
            .body_visitor
            .context
            .tcx
            .is_mir_available(self.callee_def_id)
        {
            // Get initial state from caller's state
            // We need to get all the values that may be used in callee's analysis
            // So here we get all the values that represent heap allocations

            // let init_abstract_value = self.extract_heap_value(&self.block_visitor.state);
            // TODO: try to include all states of the caller
            let init_abstract_value = self.block_visitor.state().clone();

            info!("====== Fixed-Point Algorithm Starts ======");
            debug!(
                "Initializing Fixed point iterator with abstract domain: {:?}",
                init_abstract_value
            );
            let mut body_visitor = WtoFixPointIterator::new(
                self.block_visitor.body_visitor.context,
                self.callee_def_id,
                init_abstract_value,
                self.block_visitor.body_visitor.next_fresh_variable_offset,
                self.call_stack.clone(),
            );
            body_visitor.type_visitor.actual_argument_types = self.actual_argument_types.into();
            body_visitor.type_visitor.generic_arguments = self.callee_generic_arguments;
            body_visitor.type_visitor.generic_argument_map =
                self.callee_generic_argument_map.clone();

            // Initialize initial precondition using arguments of the callee
            body_visitor.init_pre_condition(self.actual_args.to_vec());

            debug!("Running fixed point iterator");
            body_visitor.run();

            // Run the bug detector
            body_visitor.run_checker();

            // Update the fresh variable offset for the next call
            self.block_visitor.body_visitor.next_fresh_variable_offset =
                body_visitor.next_fresh_variable_offset;

            let post = body_visitor.post.clone();
            debug!("Fixed point iterator finishes, post: {:?}", post);

            // Compute the join of all the basic blocks that contain a return terminator
            let joined_state = post
                .into_iter()
                .filter(|(bb, _domain)| body_visitor.result_blocks.contains(bb))
                .map(|(_bb, domain)| domain)
                .fold1(|state1, state2| state1.join(&state2))
                .expect("panic in fold1");
            return joined_state;
        }
        // If MIR is NOT available, return default abstract domain
        // AbstractDomain::default()
        self.block_visitor.state().clone()
    }

    /// Returns the function reference part of the value, if there is one.
    fn get_func_ref(&mut self, val: &Rc<SymbolicValue>) -> Option<Rc<FunctionReference>> {
        let extract_func_ref = |c: &ConstantValue| match c {
            ConstantValue::Function(func_ref) => Some(func_ref.clone()),
            _ => None,
        };
        match &val.expression {
            Expression::CompileTimeConstant(c) => {
                // debug!("Expression::CompileTimeConstant");
                return extract_func_ref(c);
            }
            Expression::Reference(path)
            | Expression::Variable {
                path,
                var_type: ExpressionType::NonPrimitive,
            }
            | Expression::Variable {
                path,
                var_type: ExpressionType::Reference,
            } => {
                // debug!("Expression::Reference/Variable");
                let closure_ty = self
                    .block_visitor
                    .body_visitor
                    .type_visitor
                    .get_path_rustc_type(path, self.block_visitor.body_visitor.current_span);
                match closure_ty.kind() {
                    TyKind::Closure(def_id, substs) => {
                        let specialized_substs = self
                            .block_visitor
                            .body_visitor
                            .type_visitor
                            .specialize_substs(
                                substs,
                                &self
                                    .block_visitor
                                    .body_visitor
                                    .type_visitor
                                    .generic_argument_map,
                            );
                        return extract_func_ref(self.block_visitor.visit_function_reference(
                            *def_id,
                            closure_ty,
                            specialized_substs,
                        ));
                    }
                    TyKind::Ref(_, ty, _) => {
                        if let TyKind::Closure(def_id, substs) = ty.kind() {
                            let specialized_substs = self
                                .block_visitor
                                .body_visitor
                                .type_visitor
                                .specialize_substs(
                                    substs,
                                    &self
                                        .block_visitor
                                        .body_visitor
                                        .type_visitor
                                        .generic_argument_map,
                                );
                            return extract_func_ref(self.block_visitor.visit_function_reference(
                                *def_id,
                                ty,
                                specialized_substs,
                            ));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }

    pub fn get_function_post_state(&mut self) -> Option<AbstractDomain<DomainType>> {
        let fun_val = self.callee_fun_val.clone();
        if let Some(func_ref) = self.get_func_ref(&fun_val) {
            if !self.call_stack.contains(&func_ref.def_id.unwrap()) {
                self.call_stack.push(func_ref.def_id.unwrap());
                debug!("call stack {:?}", self.call_stack);
                let res = Some(self.create_function_post_state());
                return res;
            }
        }
        warn!("Failed to get_func_ref");
        None
    }

    /// If the current call is to a well known function for which we don't have a cached summary,
    /// this function will update the environment as appropriate and return true. If the return
    /// result is false, just carry on with the normal logic.
    pub fn handled_as_special_function_call(&mut self) -> bool {
        let destination_path = if let Some(dest) = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest.0))
        } else {
            None
        };
        match self.callee_known_name {
            KnownNames::VecFromRawParts => {
                self.handle_from_raw_parts();
                return true;
            }
            KnownNames::MirCheckerVerify => {
                assert!(self.actual_args.len() == 1);
                debug!("Handling special function MirCheckerVerify");
                // if self.block_visitor.body_visitor.check_for_errors {
                self.report_calls_to_special_functions();
                // }
                // self.actual_args = &self.actual_args[0..1];
                // self.handle_assume();
                return true;
            }
            KnownNames::RustDealloc => {
                return true;
            }
            KnownNames::StdPanickingBeginPanic | KnownNames::StdPanickingBeginPanicFmt => {
                self.handle_panic();
                return true;
            }
            KnownNames::StdIntoVec => {
                self.handle_into_vec();
                return true;
            }
            KnownNames::CoreOpsIndex => {
                self.handle_index();
                return true;
            }
            KnownNames::StdFrom | KnownNames::StdAsMutPtr => {
                self.handle_from();
                return true;
            }
            _ => {
                let result = self.try_to_inline_special_function();
                if !result.is_bottom() {
                    if let Some(target_path) = destination_path {
                        // let target_path = self.block_visitor.visit_place(place);
                        self.block_visitor
                            .body_visitor
                            .state
                            .update_value_at(target_path.clone(), result);
                        // let exit_condition = self.block_visitor.state.entry_condition.clone();
                        // self.block_visitor
                        //     .state
                        //     .exit_conditions
                        //     .insert(*target, exit_condition);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// If the function being called is a special function like mirai_annotations.mirai_verify or
    /// std.panicking.begin_panic then report a diagnostic or create a precondition as appropriate.
    fn report_calls_to_special_functions(&mut self) {
        match self.callee_known_name {
            KnownNames::MirCheckerVerify => {
                assert!(self.actual_args.len() == 1); // The type checker ensures this.
                let (_, cond) = &self.actual_args[0];
                // let message = self.coerce_to_string(&self.actual_args[1].1);
                let message = Rc::new(String::from("dummy message"));
                self.block_visitor.check_condition(cond, message, false);
            }
            _ => unreachable!(),
        }
    }

    /// Provides special handling of functions that have no MIR bodies or that need to access
    /// internal MIRAI state in ways that cannot be expressed in normal Rust and therefore
    /// cannot be summarized in the standard_contracts crate.
    /// Returns the result of the call, or BOTTOM if the function to call is not a known
    /// special function.
    fn try_to_inline_special_function(&mut self) -> Rc<SymbolicValue> {
        match self.callee_known_name {
            KnownNames::RustAlloc => self.handle_rust_alloc(),
            KnownNames::RustAllocZeroed => self.handle_rust_alloc(),
            KnownNames::StdMemSizeOf => self.handle_size_of(),
            _ => symbolic_value::BOTTOM.into(),
        }
    }

    // /// Removes the heap block and all paths rooted in it from the current environment.
    // fn handle_rust_dealloc(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 3);

    //     // The current environment is that that of the caller, but the caller is a standard
    //     // library function and has no interesting state to purge.
    //     // The layout path inserted below will become a side effect of the caller and when that
    //     // side effect is refined by the caller's caller, the refinement will do the purge if the
    //     // qualifier of the path is a heap block path.

    //     // Get path to the heap block to deallocate
    //     let heap_block_path = self.actual_args[0].0.clone();

    //     // Create a layout
    //     let length = self.actual_args[1].1.clone();
    //     let alignment = self.actual_args[2].1.clone();
    //     let layout = SymbolicValue::make_from(
    //         Expression::HeapBlockLayout {
    //             length,
    //             alignment,
    //             source: LayoutSource::DeAlloc,
    //         },
    //         1,
    //     );

    //     // Get a layout path and update the environment
    //     let layout_path =
    //         Path::new_layout(heap_block_path).refine_paths(&self.block_visitor.state());
    //     self.block_visitor
    //         .body_visitor
    //         .state
    //         .update_value_at(layout_path, layout);

    //     // Signal to the caller that there is no return result
    //     symbolic_value::BOTTOM.into()
    // }

    /// Returns a new heap memory block with the given byte length.
    fn handle_rust_alloc(&mut self) -> Rc<SymbolicValue> {
        assert!(self.actual_args.len() == 2);
        let length = self.actual_args[0].1.clone();
        let alignment = self.actual_args[1].1.clone();
        let tcx = self.block_visitor.body_visitor.context.tcx;
        let byte_slice = tcx.mk_slice(tcx.types.u8);
        let heap_path = Path::get_as_path(
            self.block_visitor
                .body_visitor
                .get_new_heap_block(length, alignment, byte_slice),
        );
        SymbolicValue::make_reference(heap_path)
    }

    // /// Returns a new heap memory block with the given byte length and with the zeroed flag set.
    // fn handle_rust_alloc_zeroed(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 2);
    //     let length = self.actual_args[0].1.clone();
    //     let alignment = self.actual_args[1].1.clone();
    //     let tcx = self.block_visitor.body_visitor.context.tcx;
    //     let byte_slice = tcx.mk_slice(tcx.types.u8);
    //     let heap_path = Path::get_as_path(
    //         self.block_visitor
    //             .body_visitor
    //             .get_new_heap_block(length, alignment, true, byte_slice),
    //     );
    //     SymbolicValue::make_reference(heap_path)
    // }

    // /// Sets the length of the heap block to a new value and removes index paths as necessary
    // /// if the new length is known and less than the old lengths.
    // fn handle_rust_realloc(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 4);
    //     // Get path to the heap block to reallocate
    //     let heap_block_path = Path::new_deref(self.actual_args[0].0.clone());

    //     // Create a layout
    //     let length = self.actual_args[1].1.clone();
    //     let alignment = self.actual_args[2].1.clone();
    //     let new_length = self.actual_args[3].1.clone();
    //     // We need to this to check for consistency between the realloc layout arg and the
    //     // initial alloc layout.
    //     let layout_param = SymbolicValue::make_from(
    //         Expression::HeapBlockLayout {
    //             length,
    //             alignment: alignment.clone(),
    //             source: LayoutSource::ReAlloc,
    //         },
    //         1,
    //     );
    //     // We need this to keep track of the new length
    //     let new_length_layout = SymbolicValue::make_from(
    //         Expression::HeapBlockLayout {
    //             length: new_length,
    //             alignment,
    //             source: LayoutSource::ReAlloc,
    //         },
    //         1,
    //     );

    //     // Get a layout path and update the environment
    //     let layout_path =
    //         Path::new_layout(heap_block_path).refine_paths(&self.block_visitor.state());
    //     self.block_visitor
    //         .body_visitor
    //         .state
    //         .update_value_at(layout_path.clone(), new_length_layout);
    //     let layout_path2 = Path::new_layout(layout_path);
    //     self.block_visitor
    //         .body_visitor
    //         .state
    //         .update_value_at(layout_path2, layout_param);

    //     // Return the original heap block reference as the result
    //     self.actual_args[0].1.clone()
    // }

    // /// Set the call result to an offset derived from the arguments. Does no checking.
    // fn handle_arith_offset(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 2);
    //     let base_val = &self.actual_args[0].1;
    //     let offset_val = &self.actual_args[1].1;
    //     base_val.offset(offset_val.clone())
    // }

    // /// Set the call result to an offset derived from the arguments.
    // /// Checks that the resulting offset is either in bounds or one
    // /// byte past the end of an allocated object.
    // fn handle_offset(&mut self) -> Rc<SymbolicValue> {
    //     assert!(self.actual_args.len() == 2);
    //     let base_val = &self.actual_args[0].1;
    //     let offset_val = &self.actual_args[1].1;
    //     let result = base_val.offset(offset_val.clone());

    //     let solver = &self.block_visitor.body_visitor.z3_solver;

    //     let base = Path::get_as_path(self.actual_args[0].1.clone());
    //     let base_len = Path::new_field(base, 1);
    //     let offset = self.actual_args[1].0.clone();

    //     let constraint_system =
    //         LinearConstraintSystem::from(&self.block_visitor.state().numerical_domain);
    //     for cst in &constraint_system {
    //         solver.assert(&solver.get_as_z3_expression(cst));
    //     }

    //     let mut exp = LinearExpression::default();
    //     exp = exp + base_len - offset;
    //     let cst = LinearConstraint::LessEq(exp);
    //     solver.assert(&solver.get_as_z3_expression(&cst));

    //     let solver_result = solver.solve();
    //     solver.reset();

    //     if solver_result == SmtResult::Sat {
    //         let warning = self
    //             .block_visitor
    //             .body_visitor
    //             .context
    //             .session
    //             .struct_span_warn(
    //                 self.block_visitor.body_visitor.current_span,
    //                 format!("Possible out-of-bound offset").as_str(),
    //             );
    //         self.block_visitor
    //             .body_visitor
    //             .emit_diagnostic(warning, true);
    //     } else {
    //         debug!("Proved that offset is safe!");
    //     }

    //     // if self.block_visitor.body_visitor.check_for_errors && self.function_being_analyzed_is_root() {
    //     //     self.check_offset(&result)
    //     // }
    //     result
    // }

    /// Gets the size in bytes of the type parameter T of the std::mem::size_of<T> function.
    /// Returns and unknown value of type u128 if T is not a concrete type.
    fn handle_size_of(&mut self) -> Rc<SymbolicValue> {
        assert!(self.actual_args.is_empty());
        let sym = rustc_span::Symbol::intern("T");
        let t = (self.callee_generic_argument_map.as_ref())
            .expect("std::mem::size_of must be called with generic arguments")
            .get(&sym)
            .expect("std::mem::size must have generic argument T");
        let param_env = self
            .block_visitor
            .body_visitor
            .context
            .tcx
            .param_env(self.callee_def_id);
        if let Ok(ty_and_layout) = self
            .block_visitor
            .body_visitor
            .context
            .tcx
            .layout_of(param_env.and(*t))
        {
            Rc::new((ty_and_layout.layout.size.bytes() as u128).into())
        } else {
            // SymbolicValue::make_typed_unknown(ExpressionType::U128)
            Rc::new(symbolic_value::TOP)
        }
    }

    fn handle_into_vec(&mut self) {
        assert!(self.actual_args.len() == 1);
        let source = &self.actual_args[0].0;
        let destination_path = if let Some(dest) = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest.0))
        } else {
            None
        };
        assert!(destination_path.is_some());

        let result = destination_path.as_ref().unwrap();

        let body_visitor = &mut self.block_visitor.body_visitor;
        let rtype = body_visitor
            .type_visitor
            .get_path_rustc_type(source, body_visitor.current_span);
        self.block_visitor
            .copy_or_move_elements(result.clone(), source.clone(), rtype, true);
    }

    fn handle_from_raw_parts(&mut self) {
        assert!(self.actual_args.len() == 3);
        assert!(self.destination.is_some());
        let block_visitor = &mut self.block_visitor;
        // Vec::from_raw_parts captures the ownership and passes it to the `destination`
        // So we keep track of it in `tainted_variables`

        // The source
        let source = self.args[0].clone();
        if let Some(taint_sources) = block_visitor.extract_local_from_operand(&source) {
            for local in taint_sources {
                block_visitor.body_visitor.tainted_variables.insert(local);
            }
        }

        // The destination
        block_visitor
            .body_visitor
            .tainted_variables
            .insert(self.destination.unwrap().0.local);
    }

    fn handle_panic(&mut self) {
        assert!(self.actual_args.len() == 1);
        assert!(self.destination.is_none());
        let body_visitor = &mut self.block_visitor.body_visitor;
        if !body_visitor.state.is_bottom() {
            let warning = body_visitor.context.session.struct_span_warn(
                body_visitor.current_span,
                format!("[MirChecker] Possible error: run into panic code").as_str(),
            );
            body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Panic);
        }
    }

    fn handle_from(&mut self) {
        assert!(self.actual_args.len() == 1);
        let source = &self.actual_args[0].0;
        let destination_path = if let Some(dest) = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest.0))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let result = destination_path.as_ref().unwrap();

        let body_visitor = &mut self.block_visitor.body_visitor;
        let rtype = body_visitor
            .type_visitor
            .get_path_rustc_type(source, body_visitor.current_span);
        self.block_visitor
            .copy_or_move_elements(result.clone(), source.clone(), rtype, true);
    }

    fn handle_index(&mut self) {
        assert!(self.actual_args.len() == 2);
        let destination_path = if let Some(dest) = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest.0))
        } else {
            None
        };
        assert!(destination_path.is_some());
        let state = self.block_visitor.state().clone();
        let body_visitor = &mut self.block_visitor.body_visitor;

        let array = &self.actual_args[0].0;
        let array_len = Path::new_length(array.clone()).refine_paths(&body_visitor.state);
        let array_len_val = SymbolicValue::make_from(
            Expression::Variable {
                path: array_len.clone(),
                var_type: ExpressionType::Usize,
            },
            1,
        );
        let index_val = &self.actual_args[1].1;
        let result = destination_path.as_ref().unwrap();

        let assert_checker = AssertionChecker::new(body_visitor);
        let overflow_safe_cond = SymbolicValue::make_from(
            Expression::LessThan {
                left: index_val.clone(),
                right: array_len_val,
            },
            1,
        );
        let check_result = assert_checker.check_assert_condition(overflow_safe_cond, true, &state);

        match check_result {
            CheckerResult::Safe => (),
            CheckerResult::Unsafe => {
                let error = body_visitor.context.session.struct_span_warn(
                    body_visitor.current_span,
                    format!("[MirChecker] Provably error: index out of bound",).as_str(),
                );
                body_visitor.emit_diagnostic(error, false, DiagnosticCause::Index);
                return;
            }
            CheckerResult::Warning => {
                let warning = body_visitor.context.session.struct_span_warn(
                    body_visitor.current_span,
                    format!("[MirChecker] Possible error: index out of bound").as_str(),
                );
                body_visitor.emit_diagnostic(warning, false, DiagnosticCause::Index);
            }
        }

        let source =
            Path::new_index(array.clone(), index_val.clone()).refine_paths(&body_visitor.state);
        let ref_source = SymbolicValue::make_from(Expression::Reference(source).into(), 1);
        self.block_visitor
            .body_visitor
            .state
            .update_value_at(result.clone(), ref_source);
    }

    /// Returns a list of (path, value) pairs where each path is rooted by an argument (or the result)
    /// or where the path root is a heap block reachable from an argument (or the result).
    /// Since paths are created by writes, these are side-effects.
    // OK
    fn extract_side_effects(
        &self,
        env: &AbstractDomain<DomainType>,
        argument_count: usize,
        offset: usize,
    ) -> Vec<(Rc<Path>, Rc<SymbolicValue>)> {
        let mut heap_roots: HashSet<Rc<SymbolicValue>> = HashSet::new();
        let mut result = Vec::new();
        for ordinal in 0..=argument_count {
            let root = if ordinal == 0 {
                Path::new_result()
            } else {
                Path::new_parameter(ordinal, offset)
            };

            // `path` is `result`, or `path` is rooted by `result` or parameters
            for path in env
                .get_paths_iter()
                .iter()
                .filter(|p| (ordinal == 0 && (**p) == root) || p.is_rooted_by(&root))
            {
                if let Some(value) = env.value_at(path) {
                    // Find and record heap roots in paths and values
                    // For Path, heap blocks are in `PathEnum::HeapBlock`
                    // For SymbolicValue, heap blocks are in `Expression::HeapBlock`
                    path.record_heap_blocks(&mut heap_roots);
                    value.record_heap_blocks(&mut heap_roots);
                    if let Expression::Variable { path: vpath, .. } = &value.expression {
                        if ordinal > 0 && vpath.eq(path) {
                            // The value is not an update, but just what was there at function entry.
                            // TODO: path=path, when will this happen?
                            continue;
                        }
                    }
                    // We are extracting a subset of information out of env, which has not overflowed.
                    result.push((path.clone(), value.clone()));
                }
            }
        }
        // Find path whose root is a heap block reachable from an argument (or the result)
        self.extract_reachable_heap_allocations(env, &mut heap_roots, &mut result);
        result
    }

    /// Adds roots for all new heap allocated objects that are reachable by the caller.
    /// This will modify `heap_roots` and `result`
    fn extract_reachable_heap_allocations(
        &self,
        env: &AbstractDomain<DomainType>,
        heap_roots: &mut HashSet<Rc<SymbolicValue>>,
        result: &mut Vec<(Rc<Path>, Rc<SymbolicValue>)>,
    ) {
        let mut visited_heap_roots: HashSet<Rc<SymbolicValue>> = HashSet::new();
        while heap_roots.len() > visited_heap_roots.len() {
            let mut new_roots: HashSet<Rc<SymbolicValue>> = HashSet::new();
            for heap_root in heap_roots.iter() {
                if visited_heap_roots.insert(heap_root.clone()) {
                    let root = Path::get_as_path(heap_root.clone());

                    for path in env
                        .get_paths_iter()
                        .iter()
                        // If path is a heap root or is rooted by a heap root
                        .filter(|p| (**p) == root || p.is_rooted_by(&root))
                    {
                        if let Some(value) = env.value_at(path) {
                            path.record_heap_blocks(&mut new_roots);
                            value.record_heap_blocks(&mut new_roots);
                            result.push((path.clone(), value.clone()));
                        }
                    }
                }
            }
            heap_roots.extend(new_roots.into_iter());
        }
    }

    /// Updates the current state to reflect the effects of a normal return from the function call.
    pub fn transfer_and_refine_normal_return_state(
        &mut self,
        function_post_state: &AbstractDomain<DomainType>,
        old_offset: usize,
    ) {
        self.block_visitor.body_visitor.state = function_post_state.clone();

        debug!("Start to transfer and refine normal return state");
        let destination_path = if let Some(dest) = self.destination {
            Some(self.block_visitor.get_path_for_place(&dest.0))
        } else {
            None
        };
        // let destination = self.destination.clone();
        // debug!("destination: {:?}", destination);
        if let Some(target_path) = &destination_path {
            // Assign function result to target path
            debug!("target_path: {:?}", target_path);
            let return_value_path = Path::new_result();

            let side_effects =
                self.extract_side_effects(function_post_state, self.actual_args.len(), old_offset);

            debug!("side_effects: {:?}", side_effects);

            // Transfer side effects
            if !function_post_state.is_empty() {
                // TODO
                // Effects on the heap
                // debug!("Handling side effects on the heap");
                // for (path, value) in side_effects.iter() {
                //     if path.is_rooted_by_abstract_heap_block() {
                //         let rvalue = value
                //             .clone()
                //             .refine_parameters(
                //                 self.actual_args,
                //                 self.block_visitor.body_visitor.fresh_variable_offset,
                //             )
                //             .refine_paths(&self.block_visitor.state);
                //         self.block_visitor
                //             .state
                //             .update_value_at(path.clone(), rvalue);
                //     }
                //     // check_for_early_return!(self.block_visitor.body_visitor);
                // }

                // TODO
                // Effects on the call result
                debug!("Handling side effects on call result");
                self.block_visitor.transfer_and_refine(
                    &side_effects,
                    target_path.clone(),
                    &return_value_path,
                    self.actual_args,
                );

            // Effects on the call arguments
            // debug!("Handling side effects on call arguments");
            // for (i, (target_path, _)) in self.actual_args.iter().enumerate() {
            //     let parameter_path = Path::new_parameter(i + 1);
            //     self.block_visitor.transfer_and_refine(
            //         &side_effects,
            //         target_path.clone(),
            //         &parameter_path,
            //         self.actual_args,
            //     );
            //     // check_for_early_return!(self.block_visitor.body_visitor);
            // }
            }
            // funtion_post_state is empty
            else {
                // TODO
                debug!("funtion_post_state is empty");
                // We don't know anything other than the return value type.
                // We'll assume there were no side effects and no preconditions (but check this later if possible).
                // let result_type = self
                //     .block_visitor
                //     .body_visitor
                //     .type_visitor
                //     .get_place_type(place, self.block_visitor.current_span);
                let _result_type: ExpressionType = self
                    .block_visitor
                    .body_visitor
                    .type_visitor
                    .get_path_rustc_type(target_path, self.block_visitor.body_visitor.current_span)
                    .kind()
                    .into();
                // let result = SymbolicValue::make_from(
                //     Expression::UninterpretedCall {
                //         callee: self.callee_fun_val.clone(),
                //         arguments: self
                //             .actual_args
                //             .iter()
                //             .map(|(_, arg)| arg.clone())
                //             .collect(),
                //         result_type,
                //         path: return_value_path.clone(),
                //     },
                //     1,
                // );
                let result = symbolic_value::TOP.into();
                debug!("Before updating top: {:?}", self.block_visitor.state());
                self.block_visitor
                    .body_visitor
                    .state
                    .update_value_at(return_value_path, result);
            }
        }
    }
}

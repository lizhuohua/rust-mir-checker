use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::crate_context::CrateContext;
use crate::analysis::diagnostics::{Diagnostic, DiagnosticCause};
use crate::analysis::global_context::GlobalContext;
use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::k_limits;
use crate::analysis::memory::path::{Path, PathEnum, PathRefinement, PathSelector};
use crate::analysis::memory::symbolic_value::{self, SymbolicValue, SymbolicValueTrait};
use crate::analysis::mir_visitor::block_visitor::BlockVisitor;
use crate::analysis::mir_visitor::call_visitor::CallVisitor;
use crate::analysis::mir_visitor::type_visitor::{self, TypeVisitor};
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::linear_constraint::LinearConstraintSystem;
use crate::analysis::wto::{Wto, WtoCircle, WtoVertex, WtoVisitor};
use crate::analysis::z3_solver::Z3Solver;
use crate::checker::assertion_checker::AssertionChecker;
use crate::checker::checker_trait::CheckerTrait;
use itertools::Itertools;
use log::{debug, error, warn};
use rug::Integer;
use rustc_errors::DiagnosticBuilder;
use rustc_hir::def_id::DefId;
use rustc_middle::mir;
use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Span;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::rc::Rc;

/// A wto visitor used to analyze a function
pub struct WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    // Global context
    pub context: &'a mut GlobalContext<'tcx, 'compiler>,

    // The current function's DefId
    pub def_id: DefId,

    // The current function's w.t.o
    pub wto: Wto<'tcx>,

    // Current span
    pub current_span: Span,

    // Current location
    pub current_location: mir::Location,

    // The initial state for the fixed-point algorithm
    pub init_state: AbstractDomain<DomainType>,

    // Current abstract state
    pub state: AbstractDomain<DomainType>,

    // The post-condition for each basic block
    pub post: HashMap<mir::BasicBlock, AbstractDomain<DomainType>>,

    // There may be multiple return statements, record them so we can compute the union of the return values
    pub result_blocks: HashSet<mir::BasicBlock>,

    // Helper struct to get information in Rust's type system
    pub type_visitor: TypeVisitor<'tcx>,

    // Helper struct to store information about the current crate
    pub crate_context: CrateContext<'compiler, 'tcx>,

    // For each heap allocation site, we maintain an address
    // Caveat: we assume each location only allocates once
    pub heap_addresses: HashMap<mir::Location, Rc<SymbolicValue>>,

    // Stores the tainted local variables when detecting ownership corruption
    // Variables in this set potentially acquire ownership from other allocated memory
    // So keep track of them and check whether they eventually go to terminators like `Return` or `Drop`
    // If so, then mutable shared memory are created or potential use-after-free / double-free are detected
    // We only consider `mir::Local` instead of `mir::Place` for robustness
    pub tainted_variables: HashSet<mir::Local>,

    // `Place` to `SymbolicValue` Cache, used to extract conditions when analyzing assertions
    pub place_to_abstract_value: HashMap<mir::Place<'tcx>, Rc<SymbolicValue>>,

    // The start index of variables. Because functions may return values that contain local variables, so we
    // increase the index offsets so that returned variables can be distinguished from normal local variables
    pub fresh_variable_offset: usize,

    // The fresh variable offset used for the next call
    pub next_fresh_variable_offset: usize,

    // The call stack, used to detect recursive calls
    pub call_stack: Vec<DefId>,

    // The Z3 SMT solver
    pub z3_solver: Z3Solver,

    // Buffered diagnostics
    pub buffered_diagnostics: Vec<Diagnostic<'compiler>>,
}

impl<'tcx, 'a, 'compiler, DomainType> WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// The offset that we add to `fresh_variable_offset` when calling functions
    pub const FRESH_VARIABLE_OFFSET: usize = 1000000;

    /// Create a new w.t.o visitor for a given w.t.o and its initial state
    pub fn new(
        context: &'a mut GlobalContext<'tcx, 'compiler>,
        def_id: DefId,
        init_state: AbstractDomain<DomainType>,
        fresh_variable_offset: usize,
        call_stack: Vec<DefId>,
    ) -> Self {
        let wto = context.get_wto(def_id);
        let type_visitor = TypeVisitor::new(def_id, wto.get_mir().clone(), context.tcx);

        Self {
            current_span: rustc_span::DUMMY_SP,
            current_location: mir::Location::START,
            context,
            def_id,
            init_state,
            wto,
            state: AbstractDomain::default(),
            post: HashMap::new(),
            result_blocks: HashSet::new(),
            type_visitor,
            crate_context: CrateContext::default(),
            heap_addresses: HashMap::new(),
            tainted_variables: HashSet::new(),
            place_to_abstract_value: HashMap::new(),
            fresh_variable_offset,
            next_fresh_variable_offset: fresh_variable_offset + Self::FRESH_VARIABLE_OFFSET,
            call_stack,
            z3_solver: Z3Solver::default(),
            buffered_diagnostics: vec![],
        }
    }

    /// Run analysis
    pub fn run(&mut self) {
        for comp in self.wto.components() {
            self.visit_component(&comp);
        }
    }

    /// Initialize arguments when analyzing a function
    pub fn init_pre_condition(&mut self, actual_args: Vec<(Rc<Path>, Rc<SymbolicValue>)>) {
        for (i, arg) in actual_args.iter().enumerate() {
            // Initialize callee's arguments using caller's values
            // So callee's paths should add an offset to distinguish them from caller's paths
            let new_path = Path::new_parameter(i + 1, self.fresh_variable_offset);
            self.init_state.update_value_at(new_path, arg.1.clone());
        }
        debug!("Initializing pre condition: {:?}", self.init_state);
    }

    /// Run bug detectors
    pub fn run_checker(&mut self) {
        // Do not check functions that have already been checked
        if self.context.checked_def_ids.contains(&self.def_id) {
            return;
        }

        self.context.checked_def_ids.insert(self.def_id);

        let mut checker = AssertionChecker::<DomainType>::new(self);
        checker.run();

        // Store diagnostic messages for this function
        self.context
            .diagnostics_for
            .insert(self.def_id, self.buffered_diagnostics.clone());

        // Cancel the buffered diagnostics because they have been copied into global context
        // If not, the compiler will emit a bug when dropping them
        for diagnostic in &mut self.buffered_diagnostics {
            diagnostic.cancel();
        }
    }

    pub fn get_exit_state(&self) -> Option<AbstractDomain<DomainType>> {
        self.post
            .clone()
            .into_iter()
            .filter(|(bb, _domain)| self.result_blocks.contains(bb))
            .map(|(_bb, domain)| domain)
            .fold1(|state1, state2| state1.join(&state2))
    }

    pub fn init_promote_constants(&mut self)
    where
        DomainType: ApronDomainType,
        ApronAbstractDomain<DomainType>: GetManagerTrait,
    {
        debug!("Start initializing promoted constants");
        let mut environment = AbstractDomain::default();

        // For each promoted constant's MIR
        // Note that promoted MIR does not have its DefId
        for (ordinal, constant_mir) in self
            .context
            .tcx
            .promoted_mir(self.def_id) // Get promoted constants from the current function
            .iter()
            .enumerate()
        {
            debug!("Get promoted MIR {}: {:?}", ordinal, constant_mir);
            // The type of the promoted constant
            let result_rustc_type = constant_mir.local_decls[mir::Local::from(0usize)].ty;

            let mut wto_visitor = WtoFixPointIterator::new(
                self.context,
                self.def_id,
                AbstractDomain::default(),
                0,
                vec![],
            );
            let promoted_constant_wto = Wto::new(constant_mir);
            debug!("promoted constant wto: {:?}", promoted_constant_wto);
            // Substitute def_id's wto with promoted constant's wto
            wto_visitor.wto = promoted_constant_wto;
            wto_visitor.type_visitor.mir = constant_mir.clone();
            wto_visitor.run();

            // self.visit_promoted_constants_block();

            // Get the states of the `return` statements
            if let Some(exit_environment) = wto_visitor.get_exit_state() {
                debug!("Get exit state for promoted MIR: {:?}", exit_environment);
                //     self.current_environment = exit_environment.clone();

                // The path of the promoted constant in the promoted MIR
                let mut result_root: Rc<Path> = Path::new_result();
                // The path of the promoted constant in the current function's MIR
                let mut promoted_root: Rc<Path> =
                    Rc::new(PathEnum::PromotedConstant { ordinal }.into());

                // If the promoted constant is a pointer/reference to a collection type
                // We need to also create its length
                if wto_visitor
                    .type_visitor
                    .starts_with_slice_pointer(result_rustc_type.kind())
                {
                    // Get the length value from the promoted MIR
                    let source_length_path = Path::new_length(result_root.clone());
                    let length_val = exit_environment
                        .value_at(&source_length_path)
                        .expect("collection to have a length");
                    // Store the length value
                    let target_length_path = Path::new_length(promoted_root.clone());
                    environment.update_value_at(target_length_path, length_val.clone());
                    // For collection types, the paths to the value need an additional field `0`
                    promoted_root = Path::new_field(promoted_root, 0);
                    result_root = Path::new_field(result_root, 0);
                }
                // Get promoted constant value from the promoted MIR
                let value = wto_visitor
                    .lookup_path_and_refine_result(result_root.clone(), result_rustc_type);
                match &value.expression {
                    // If the promoted constant is a heap allocation
                    Expression::HeapBlock { .. } => {
                        let heap_root: Rc<Path> = Rc::new(
                            PathEnum::HeapBlock {
                                value: value.clone(),
                            }
                            .into(),
                        );
                        for (path, value) in exit_environment
                            .symbolic_domain
                            .value_map
                            .iter()
                            .filter(|(p, _)| p.is_rooted_by(&heap_root))
                        {
                            // Put all the values in promoted MIR that are rooted by the heap allocation in the environment
                            environment.update_value_at(path.clone(), value.clone());
                        }
                        // Put the promoted constant value itself in the environment
                        environment.update_value_at(promoted_root.clone(), value.clone());
                    }
                    Expression::Reference(local_path) => {
                        wto_visitor.promote_reference(
                            &mut environment,
                            result_rustc_type,
                            &promoted_root,
                            local_path,
                            ordinal,
                        );
                    }
                    _ => {
                        for (path, value) in exit_environment
                            .symbolic_domain
                            .value_map
                            .iter()
                            .filter(|(p, _)| p.is_rooted_by(&result_root))
                        {
                            // For all the paths that are rooted by the promoted value in promoted MIR
                            // Replace the root with the promoted path used in the current function's MIR
                            let promoted_path =
                                path.replace_root(&result_root, promoted_root.clone());
                            environment.update_value_at(promoted_path, value.clone());
                        }
                        if let Expression::Variable { .. } = &value.expression {
                            // The constant is a stack allocated struct. No need for a separate entry.
                        } else {
                            environment.update_value_at(promoted_root.clone(), value.clone());
                        }
                    }
                }
            }
        }
        debug!(
            "Before join, init: {:?}, environment: {:?}",
            self.init_state, environment
        );
        self.init_state = self.init_state.meet(&environment);
        debug!("After meet, environment: {:?}", self.init_state);
        debug!(
            "Finish initializing promoted constants, init_state: {:?}",
            self.init_state
        );
    }

    fn promote_reference(
        &mut self,
        environment: &mut AbstractDomain<DomainType>,
        result_rustc_type: Ty<'tcx>,
        promoted_root: &Rc<Path>,
        local_path: &Rc<Path>,
        mut ordinal: usize,
    ) where
        DomainType: ApronDomainType,
        ApronAbstractDomain<DomainType>: GetManagerTrait,
    {
        debug!("In promote_reference, state: {:?}", self.state);
        let target_type = type_visitor::get_target_type(result_rustc_type);
        // If the promoted constant is a reference/pointer to a primitive value (Not NonPrimitive or Reference)
        if ExpressionType::from(target_type.kind()).is_primitive() {
            // Kind of weird, but seems to be generated for debugging support.
            // Move the value into a path, so that we can drop the reference to the soon to be dead local.
            let target_value = self
                .state
                .value_at(local_path)
                .expect("expect reference target to have a value");
            let value_path = Path::get_as_path(target_value.clone());
            let promoted_value = SymbolicValue::make_from(Expression::Reference(value_path), 1);
            environment.update_value_at(promoted_root.clone(), promoted_value);
        } else if let TyKind::Ref(_, ty, _) = target_type.kind() {
            // Promoting a reference to a reference.
            ordinal += 99;
            let value_path: Rc<Path> = Rc::new(PathEnum::PromotedConstant { ordinal }.into());
            self.promote_reference(environment, ty, &value_path, local_path, ordinal);
            let promoted_value = SymbolicValue::make_from(Expression::Reference(value_path), 1);
            environment.update_value_at(promoted_root.clone(), promoted_value);
        } else {
            // A composite value needs to get to get promoted to the heap
            // in order to propagate it via function summaries.
            let byte_size = self.type_visitor.get_type_size(target_type);
            let byte_size_value = self.get_u128_const_val(byte_size as u128);
            let elem_size = self
                .type_visitor
                .get_type_size(type_visitor::get_element_type(target_type));
            let alignment: Rc<SymbolicValue> = Rc::new(
                (match elem_size {
                    0 => 1,
                    1 | 2 | 4 | 8 => elem_size,
                    _ => 8,
                } as u128)
                    .into(),
            );
            let heap_value = self.get_new_heap_block(byte_size_value, alignment, target_type);
            let heap_root = Path::get_as_path(heap_value);
            // let layout_path = Path::new_layout(heap_root.clone());
            // let layout_value = self
            //     .state
            //     .value_at(&layout_path)
            //     .expect("new heap block should have a layout");
            // environment.update_value_at(layout_path, layout_value.clone());
            // for (path, value) in self
            //     .state
            //     .symbolic_domain
            //     .value_map
            //     .iter()
            //     .filter(|(p, _)| (*p) == local_path || p.is_rooted_by(local_path))
            // {
            //     debug!("Find: path: {:?}, value: {:?}", path, value);
            //     let renamed_path = path.replace_root(local_path, heap_root.clone());
            //     environment.update_value_at(renamed_path, value.clone());
            // }

            for path in self.state.get_paths_iter() {
                if let Some(value) = self.state.value_at(&path) {
                    if (&path) == local_path || path.is_rooted_by(local_path) {
                        debug!("Find: path: {:?}, value: {:?}", path, value);
                        let renamed_path = path.replace_root(local_path, heap_root.clone());
                        environment.update_value_at(renamed_path, value.clone());
                    }
                }
            }

            let thin_pointer_to_heap = SymbolicValue::make_reference(heap_root);
            if type_visitor::is_slice_pointer(target_type.kind()) {
                let promoted_thin_pointer_path = Path::new_field(promoted_root.clone(), 0);
                environment.update_value_at(promoted_thin_pointer_path, thin_pointer_to_heap);
                let length_value = self
                    .state
                    .value_at(&Path::new_length(local_path.clone()))
                    .unwrap_or_else(|| unreachable!("promoted constant slice source is expected to have a length value, see source at {:?}", self.current_span))
                    .clone();
                let length_path = Path::new_length(promoted_root.clone());
                environment.update_value_at(length_path, length_value);
            } else {
                environment.update_value_at(promoted_root.clone(), thin_pointer_to_heap);
            }
        }
        debug!("Finish promote reference, environment: {:?}", environment);
    }

    pub fn get_new_heap_block(
        &mut self,
        _length: Rc<SymbolicValue>,
        _alignment: Rc<SymbolicValue>,
        // is_zeroed: bool,
        ty: Ty<'tcx>,
    ) -> Rc<SymbolicValue> {
        let addresses = &mut self.heap_addresses;
        let constants = &mut self.crate_context.constant_value_cache;
        let block = addresses
            .entry(self.current_location)
            .or_insert_with(|| SymbolicValue::make_from(constants.get_new_heap_block(), 1))
            .clone();
        let block_path = Path::get_as_path(block.clone());
        self.type_visitor
            .path_ty_cache
            .insert(block_path.clone(), ty);
        // let layout_path = Path::new_layout(block_path);
        // let layout = SymbolicValue::make_from(
        //     Expression::HeapBlockLayout {
        //         length,
        //         alignment,
        //         source: LayoutSource::Alloc,
        //     },
        //     1,
        // );
        // self.state.update_value_at(layout_path, layout);
        block
    }

    // TODO: check this
    // When executing: path: local_3, result type: &mut i32, where local_3 is a &(local_1), this function returns local1: Reference
    // It should return &(local_1), fixed.
    // When executing: path: <heap0>, result type: [u32; 5], this function returns <heap0>: NonPrimitive
    // It should return <heap0> directly
    pub fn lookup_path_and_refine_result(
        &mut self,
        path: Rc<Path>,
        result_rustc_type: Ty<'tcx>,
    ) -> Rc<SymbolicValue> {
        debug!(
            "lookup_path_and_refine_result: {:?}, result type: {:?}",
            path, result_rustc_type
        );
        let result_type: ExpressionType = (result_rustc_type.kind()).into();
        match &path.value {
            PathEnum::Alias { value } => {
                return value.clone();
            }
            // PathEnum::HeapBlock { value } => {
            //     return value.clone();
            // }
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } if matches!(selector.as_ref(), PathSelector::Deref) => {
                let path = Path::new_qualified(
                    qualifier.clone(),
                    Rc::new(PathSelector::Index(Rc::new(0u128.into()))),
                );
                if self.state.value_at(&path).is_some() {
                    let refined_val = self.lookup_path_and_refine_result(path, result_rustc_type);
                    if !refined_val.is_bottom() {
                        return refined_val;
                    }
                }
            }
            _ => {}
        }
        let refined_val = {
            let top = symbolic_value::TOP.into();
            self.state.value_at(&path).unwrap_or(top)
        };
        debug!("refined_val: {:?}", refined_val);
        let result = if refined_val.is_top() {
            // Not found locally, so try statics.
            if path.path_length() < k_limits::MAX_PATH_LENGTH {
                let mut result = None;
                if let PathEnum::QualifiedPath {
                    qualifier,
                    selector,
                    ..
                } = &path.value
                {
                    match selector.as_ref() {
                        PathSelector::Deref | PathSelector::Index(..) => {
                            if let PathSelector::Index(index_val) = selector.as_ref() {
                                result = self.lookup_weak_value(qualifier, index_val);
                            }
                            // If failed to lookup fat pointer && weak value, but the type is integer
                            if result.is_none() && result_type.is_integer() {
                                let _qualifier_val = self.lookup_path_and_refine_result(
                                    qualifier.clone(),
                                    ExpressionType::NonPrimitive.as_rustc_type(self.context.tcx),
                                );
                            }
                        }
                        PathSelector::Discriminant => {
                            let ty = type_visitor::get_target_type(
                                self.type_visitor
                                    .get_path_rustc_type(qualifier, self.current_span),
                            );
                            match ty.kind() {
                                TyKind::Adt(..) if ty.is_enum() => {}
                                TyKind::Generator(..) => {}
                                _ => {
                                    result = Some(self.get_u128_const_val(0));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                debug!("result: {:?}", result);
                result.unwrap_or_else(|| {
                    let result = match &path.value {
                        PathEnum::HeapBlock { value: _ } => {
                            SymbolicValue::make_typed_unknown(result_type.clone())
                        }
                        _ => SymbolicValue::make_from(
                            Expression::Variable {
                                path: path.clone(),
                                var_type: result_type.clone(),
                            },
                            1,
                        ),
                    };
                    if result_type != ExpressionType::NonPrimitive {
                        self.state.update_value_at(path, result.clone());
                    }
                    result
                })
            } else {
                // SymbolicValue::make_typed_unknown(result_type.clone())
                let result = match path.value {
                    PathEnum::LocalVariable { .. } => refined_val,
                    _ => SymbolicValue::make_typed_unknown(result_type.clone()),
                };
                if result_type != ExpressionType::NonPrimitive {
                    self.state.update_value_at(path, result.clone());
                }
                result
            }
        }
        // Found in local, just return
        else {
            debug!("refined_val: {:?}", refined_val);
            refined_val
        };

        debug!("Result: {:?}", result);

        if result_type != ExpressionType::Reference
            && result.expression.infer_type() == ExpressionType::Reference
        {
            result.dereference(result_type)
        } else {
            result
        }
    }

    pub fn import_static(&mut self, path: Rc<Path>) -> Rc<Path> {
        debug!("In import_static, path: {:?}", path);
        if let PathEnum::StaticVariable {
            def_id,
            summary_cache_key,
            expression_type,
        } = &path.value
        {
            if self.state.value_at(&path).is_some() {
                return path;
            }
            self.state.update_value_at(
                path.clone(),
                SymbolicValue::make_typed_unknown(expression_type.clone()),
            );
            self.import_def_id_as_static(&path, *def_id, summary_cache_key);
        }
        path
    }

    fn import_def_id_as_static(
        &mut self,
        _path: &Rc<Path>,
        def_id: Option<DefId>,
        _summary_cache_key: &Rc<String>,
    ) {
        debug!("In import_def_id_as_static");
        let environment_before_call = self.state.clone();
        // let saved_analyzing_static_var = self.analyzing_static_var;
        // self.analyzing_static_var = true;
        let mut block_visitor;
        // let summary;
        if let Some(def_id) = def_id {
            if self.call_stack.contains(&def_id) {
                return;
            }
            let generic_args = self.crate_context.substs_cache.get(&def_id).cloned();
            let callee_generic_argument_map = if let Some(generic_args) = generic_args {
                self.type_visitor
                    .get_generic_arguments_map(def_id, generic_args, &[])
            } else {
                None
            };
            let ty = self.context.tcx.type_of(def_id);
            let func_const = self
                .crate_context
                .constant_value_cache
                .get_function_constant_for(
                    def_id,
                    ty,
                    generic_args,
                    self.context.tcx,
                    &mut self.crate_context.known_names_cache,
                    // &mut self.cv.summary_cache,
                )
                .clone();
            block_visitor = BlockVisitor::new(self, environment_before_call);
            let mut call_visitor = CallVisitor::new(
                &mut block_visitor,
                def_id,
                generic_args,
                callee_generic_argument_map,
                // environment_before_call,
                func_const,
            );
            let _func_ref = call_visitor
                .callee_func_ref
                .clone()
                .expect("CallVisitor::new should guarantee this");

            debug!("Executing call visitor for static variable...");
            // Run the call visitor and get post states
            let function_post_state = call_visitor
                .get_function_post_state()
                .unwrap_or_else(AbstractDomain::default);

            debug!(
                "Finish call visitor, get function post state {:?}",
                function_post_state
            );
            debug!(
                "Before handling side-effects, pre env {:?}",
                call_visitor.block_visitor.state()
            );
            call_visitor.transfer_and_refine_normal_return_state(&function_post_state, 0);
            debug!(
                "After handling side-effects, post env {:?}",
                call_visitor.block_visitor.state()
            );
        };
    }

    fn lookup_weak_value(
        &mut self,
        key_qualifier: &Rc<Path>,
        _key_index: &Rc<SymbolicValue>,
    ) -> Option<Rc<SymbolicValue>> {
        // TODO: Do we need to directly access the symbolic domain?
        for (path, value) in self.state.symbolic_domain.value_map.iter() {
            if let PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } = &path.value
            {
                if let PathSelector::Slice(..) = selector.as_ref() {
                    if value.expression.infer_type().is_primitive() && key_qualifier.eq(qualifier) {
                        // This is the supported case for arrays constructed via a repeat expression.
                        // We assume that index is in range since that has already been checked.
                        // todo: deal with the case where there is another path that aliases the slice.
                        // i.e. a situation that arises if a repeat initialized array has been updated
                        // with an index that is not an exact match for key_index.
                        return Some(value.clone());
                    }
                }
                // todo: deal with PathSelector::Index when there is a possibility that
                // key_index might match it at runtime.
            }
        }
        None
    }

    // TODO: do we need to distinguish signed and unsigned integers? And how to use rug to implement it?
    pub fn get_i128_const_val(&mut self, val: i128) -> Rc<SymbolicValue> {
        Rc::new(ConstantValue::Int(Integer::from(val)).into())
    }

    pub fn get_u128_const_val(&mut self, val: u128) -> Rc<SymbolicValue> {
        Rc::new(ConstantValue::Int(Integer::from(val)).into())
    }

    /// Try to get the symbol name of a variable in debug information
    /// If failed to find the symbol, return a string according to its `Debug` trait implementation
    pub fn get_var_name(&self, operand: &mir::Operand<'tcx>) -> String {
        for var_info in &self.wto.get_mir().var_debug_info {
            match var_info.value {
                mir::VarDebugInfoContents::Place(place1) => match operand {
                    mir::Operand::Copy(place2) | mir::Operand::Move(place2) => {
                        if place1 == *place2 {
                            return var_info.name.to_ident_string();
                        }
                        return format!("{:?}", operand);
                    }
                    _ => return format!("{:?}", operand),
                },
                mir::VarDebugInfoContents::Const(constant1) => match operand {
                    mir::Operand::Constant(constant2) => {
                        if constant1 == **constant2 {
                            return var_info.name.to_ident_string();
                        }
                        return format!("{:?}", operand);
                    }
                    _ => return format!("{:?}", operand),
                },
            }
        }
        // Get here if not found
        format!("{:?}", operand)
    }

    /// Recover the variable name for each assert message
    /// This is used to pretty print the diagnostic messages
    pub fn recover_var_name(&self, assert_kind: &mir::AssertKind<mir::Operand<'tcx>>) -> String {
        use mir::AssertKind::*;
        use mir::BinOp;

        // The following code is adapted from the original implementation of the `Debug` trait for `AssertKind`
        match assert_kind {
            BoundsCheck { ref len, ref index } => format!(
                "index out of bounds: the length is {:?} but the index is {:?}",
                self.get_var_name(len),
                self.get_var_name(index)
            ),
            OverflowNeg(op) => format!(
                "attempt to negate `{:#?}`, which would overflow",
                self.get_var_name(op)
            ),
            DivisionByZero(op) => {
                format!("attempt to divide `{:#?}` by zero", self.get_var_name(op))
            }
            RemainderByZero(op) => format!(
                "attempt to calculate the remainder of `{:#?}` with a divisor of zero",
                self.get_var_name(op)
            ),
            Overflow(BinOp::Add, l, r) => {
                format!(
                    "attempt to compute `{:#?} + {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Sub, l, r) => {
                format!(
                    "attempt to compute `{:#?} - {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Mul, l, r) => {
                format!(
                    "attempt to compute `{:#?} * {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Div, l, r) => {
                format!(
                    "attempt to compute `{:#?} / {:#?}`, which would overflow",
                    self.get_var_name(l),
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Rem, l, r) => format!(
                "attempt to compute the remainder of `{:#?} % {:#?}`, which would overflow",
                self.get_var_name(l),
                self.get_var_name(r)
            ),
            Overflow(BinOp::Shr, _, r) => {
                format!(
                    "attempt to shift right by `{:#?}`, which would overflow",
                    self.get_var_name(r)
                )
            }
            Overflow(BinOp::Shl, _, r) => {
                format!(
                    "attempt to shift left by `{:#?}`, which would overflow",
                    self.get_var_name(r)
                )
            }
            _ => format!("{}", assert_kind.description()),
        }
    }

    pub fn emit_diagnostic(
        &mut self,
        mut diagnostic_builder: DiagnosticBuilder<'compiler>,
        is_memory_safety: bool,
        cause: DiagnosticCause,
    ) {
        use rustc_span::hygiene::{ExpnData, ExpnKind, MacroKind};
        if let [span] = &diagnostic_builder.span.primary_spans() {
            if let Some(ExpnData {
                kind: ExpnKind::Macro(MacroKind::Derive, ..),
                ..
            }) = span.source_callee()
            {
                info!("derive macro has warning: {:?}", diagnostic_builder);
                diagnostic_builder.cancel();
                return;
            }
        }
        let diagnostic = Diagnostic::new(diagnostic_builder, is_memory_safety, cause);
        self.buffered_diagnostics.push(diagnostic);
    }

    // The following are private methods

    /// Execute block visitor to analyze a basic block
    fn analyze_basic_block(&mut self, bb: mir::BasicBlock, pre: AbstractDomain<DomainType>) {
        debug!("###########################################################################");
        debug!("Analyzing basic block: {:?}", bb);
        debug!("Pre-Condition for {:?}: {:?}", bb, pre);
        let post;
        if !pre.is_bottom() {
            let mut visitor = BlockVisitor::new(self, pre);
            visitor.visit_basic_block(bb);
            post = &self.state;
        } else {
            debug!("The precondition is bottom, ignore the analysis for this block");
            post = &pre;
        }
        debug!("Finish analyzing basic block: {:?}", bb);
        debug!("Post-Condition for {:?}: {:?}", bb, post);
        debug!("Exit condition {:?}: {:?}", bb, post.exit_conditions);
        self.post.insert(bb, post.clone());
        debug!("###########################################################################\n");
    }

    /// Perform widening if the iteration counter exceeds `widening_delay`
    fn extrapolate(
        &mut self,
        circle: &WtoCircle,
        before: AbstractDomain<DomainType>,
        after: AbstractDomain<DomainType>,
    ) -> AbstractDomain<DomainType> {
        let iteration = circle.get_iter_num();
        let widening_delay = self.context.analysis_options.widening_delay;
        let bb = circle.head().node(); // Get head basic block from circle

        if iteration <= widening_delay {
            // We haven't reached the threshold for widening, so we just execute lub
            before.join(&after)
        } else {
            debug!("Widening for {:?} at iteration: {}", bb, iteration);
            // We have reached the threshold for widening, execute widening
            before.widening_with(&after)
        }
    }

    /// Perform narrowing according to the iteration counter
    fn refine(
        &mut self,
        circle: &WtoCircle,
        before: AbstractDomain<DomainType>,
        after: AbstractDomain<DomainType>,
    ) -> AbstractDomain<DomainType> {
        let iteration = circle.get_iter_num();

        if iteration == 1 {
            // Make sure it will converge
            debug!(
                "Narrowing for {:?} at iteration: {}, use `meet` to guarantee convergence",
                circle.head().node(),
                iteration
            );
            before.meet(&after)
        } else {
            debug!(
                "Narrowing for {:?} at iteration: {}",
                circle.head().node(),
                iteration
            );
            before.narrowing_with(&after)
        }
    }

    /// Merge all the predecessors' states
    fn get_state_from_predecessors(&mut self, bb: mir::BasicBlock) -> AbstractDomain<DomainType> {
        debug!("Start merging state from predecessors");
        let pred_states: Vec<AbstractDomain<DomainType>> =
            // For all predecessors of bb
            self.wto.get_mir().predecessors()[bb]
                .iter()
                .filter_map(|pred_bb| {
                    // For a predecessor pred_bb, get the post condition
                    if let Some(pred_state) = self.post.get(pred_bb) {
                        let mut pred_state = pred_state.clone();
                        debug!("Get state from {:?}: {:?}", pred_bb, pred_state);
                        // If pred_bb has exit conditions that need to be propagated to bb, add constraints in pred_state
                        if let Some(pred_exit_condition) = pred_state.exit_conditions.get(&bb) {
                            debug!("Get exit condition for {:?}: {:?}", bb, pred_exit_condition);
                            debug!("State before adding constraint: {:?}", pred_state);
                            match LinearConstraintSystem::try_from(pred_exit_condition.clone()){
                                Ok(linear_constraint_system) => pred_state.numerical_domain.add_constraints(linear_constraint_system),
                                Err(e) => error!("{}", e),
                            }
                            debug!("State after adding constraint: {:?}", pred_state);
                        }
                        Some(pred_state)
                    } else {
                        None
                    }
                })
                .collect();
        // Merge states using the join operator
        let joined_state = pred_states
            .into_iter()
            .fold1(|state1, state2| state1.join(&state2))
            .expect("Panic while merging states using fold1");
        debug!("Merged state: {:?}", joined_state);
        joined_state
    }
}

/// Implement `visit_vertex` and `visit_circle`
impl<'tcx, 'a, 'compiler, DomainType> WtoVisitor
    for WtoFixPointIterator<'tcx, 'a, 'compiler, DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Visit a node in w.t.o
    fn visit_vertex(&mut self, vertex: &WtoVertex) {
        let bb = vertex.node();
        // If bb is the entry block (block ID is 0), initialize precondition as init state
        let pre = if vertex.is_entry() {
            self.init_state.clone()
        } else {
            // Otherwise, compute the disjunction of all the predecessors' post conditions
            self.get_state_from_predecessors(bb)
        };
        // self.set_pre(bb, pre.clone());

        // Now analyze this node
        self.analyze_basic_block(bb, pre);
    }

    /// Visit a circle in w.t.o, the analysis will only proceed if the circle reaches its fixed-point
    fn visit_circle(&mut self, circle: &WtoCircle) {
        let head = circle.head();
        let head_bb = head.node();
        debug!("Analyzing loop {:?} with head: {:?}", circle, head);
        // First, find out the precondition of the head node of this circle
        let mut pre = if head.is_entry() {
            // If the head of this circle is the entry block (FIXME: Is it possible?)
            warn!("The head of a circle is the entry block");
            self.init_state.clone()
        } else {
            // Compute the disjunction of the predecessors' post conditions
            self.get_state_from_predecessors(head_bb)
        };

        // Perform the fixed-point algorithm
        loop {
            // Increment iteration counter
            circle.inc_iter_num();

            // Analyze the head basic block
            self.analyze_basic_block(head_bb, pre.clone());

            // Analyze the rest blocks in the body
            for comp in circle {
                self.visit_component(&comp);
            }

            // Check whether fixed-point is reached
            let new_pre = self.get_state_from_predecessors(head_bb);
            if new_pre.leq(&pre) {
                debug!("A Fixed-Point has been reached!");
                break;
            } else {
                debug!("Fixed point is not reached because `new_pre <= pre` does not hold");
                debug!("new_pre: {:?}", new_pre);
                debug!("pre:     {:?}", pre);
                // Fixed-point is not reached, try widening if iteration counter exceeds the threshold
                pre = self.extrapolate(circle, pre, new_pre);
            }
        }

        // Fixed-point is reached, try narrowing
        // Narrowing is not guaranteed to converge in general, so we simply iterate at most `narrowing_iteration` times
        let narrowing_iteration = self.context.analysis_options.narrowing_iteration;
        if narrowing_iteration != 0 {
            for _i in 1..narrowing_iteration + 1 {
                // Narrowing: analyze again in order to get a better result
                // Analyze the head basic block
                self.analyze_basic_block(head_bb, pre.clone());

                // Analyze the rest blocks in the body
                for comp in circle {
                    self.visit_component(&comp);
                }

                // Check whether fixed-point is reached
                let new_pre = self.get_state_from_predecessors(head_bb);

                // Note that here the order is different from the above fixed-point check
                if pre.leq(&new_pre) {
                    // No need for refinement
                    // TODO: do we need to restore post condition here?
                    break;
                } else {
                    pre = self.refine(circle, pre, new_pre);
                }
            }
        }
    }
}

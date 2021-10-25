// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
//

use super::expression::{Expression, ExpressionType};
use super::symbolic_value::{self, SymbolicValue, SymbolicValueRefinement, SymbolicValueTrait};
use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::memory::utils;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Represent a memory location as a path
#[derive(Clone, Eq, Ord, PartialOrd)]
pub struct Path {
    pub value: PathEnum,
    hash: u64,
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.value.fmt(f)
    }
}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Path) -> bool {
        self.hash == other.hash && self.value == other.value
    }
}

impl From<PathEnum> for Path {
    fn from(value: PathEnum) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Path {
            value,
            hash: hasher.finish(),
        }
    }
}

impl Path {
    /// Returns a qualified path of the form root.selectors[0].selectors[1]...
    // TODO: This is only used in handling weak updates
    // See whether we really need weak updates, if not, remove this
    pub fn add_selectors(root: &Rc<Path>, selectors: &[Rc<PathSelector>]) -> Rc<Path> {
        let mut result = root.clone();
        for selector in selectors.iter() {
            result = Path::new_qualified(result, selector.clone());
        }
        result
    }

    /// Requires an abstract value that is an AbstractHeapAddress expression and
    /// returns a path can be used as the root of paths that define the heap value.
    pub fn get_as_path(value: Rc<SymbolicValue>) -> Rc<Path> {
        Rc::new(match &value.expression {
            Expression::HeapBlock { .. } => PathEnum::HeapBlock { value }.into(),
            Expression::Reference(path)
            | Expression::Variable { path, .. }
            | Expression::Numerical(path)
            | Expression::Widen { path, .. } => path.as_ref().clone(),
            _ => PathEnum::Alias { value }.into(),
        })
    }
}

/// A path represents a left hand side expression.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PathEnum {
    /// A path to a value that is not stored at a single memory location.
    /// For example, a compile time constant will not have a location.
    /// Another example is a conditional value with is either a parameter or a local variable,
    /// depending on a condition.
    /// In general, such a paths is needed when the value is an argument to a function call and
    /// the corresponding parameter shows up in the function summary as part of a path (usually a
    /// qualifier). In order to replace the parameter with the argument value, we need a path that
    /// wraps the argument value. When the value thus wrapped contains a reference to another path
    /// (or paths), the wrapper path is an alias to those paths.
    Alias { value: Rc<SymbolicValue> },

    /// A dynamically allocated memory block.
    HeapBlock { value: Rc<SymbolicValue> },

    /// locals [arg_count+1..] are the local variables and compiler temporaries.
    LocalVariable { ordinal: usize },

    /// locals [1..=arg_count] are the parameters
    Parameter { ordinal: usize },

    /// local 0 is the return value temporary
    Result,

    /// The name is a summary cache key string.
    StaticVariable {
        /// The crate specific key that is used to identify the function in the current crate.
        /// This is not available for functions returned by calls to functions from other crates,
        /// since the def id the other crates use have no meaning for the current crate.
        def_id: Option<DefId>,
        /// The key to use when retrieving a summary for the static variable from the summary cache.
        summary_cache_key: Rc<String>,
        /// The type to use when the static variable value is not yet available.
        expression_type: ExpressionType,
    },

    /// The ordinal is an index into a method level table of MIR bodies.
    PromotedConstant { ordinal: usize },

    /// The qualifier denotes some reference, struct, or collection.
    /// The selector denotes a de-referenced item, field, or element, or slice.
    QualifiedPath {
        length: usize,
        qualifier: Rc<Path>,
        selector: Rc<PathSelector>,
    },
}

impl Debug for PathEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PathEnum::Alias { value } => f.write_fmt(format_args!("alias_{:?}", value)),
            PathEnum::HeapBlock { value } => f.write_fmt(format_args!("<{:?}>", value)),
            PathEnum::LocalVariable { ordinal } => f.write_fmt(format_args!("local_{}", ordinal)),
            PathEnum::Parameter { ordinal } => f.write_fmt(format_args!("param_{}", ordinal)),
            PathEnum::Result => f.write_str("result"),
            PathEnum::StaticVariable {
                summary_cache_key, ..
            } => summary_cache_key.fmt(f),
            PathEnum::PromotedConstant { ordinal } => {
                f.write_fmt(format_args!("constant_{}", ordinal))
            }
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } => f.write_fmt(format_args!("{:?}.{:?}", qualifier, selector)),
        }
    }
}

impl Path {
    /// True if path qualifies root, or another qualified path rooted by root.
    // Used to determine whether a path depends on another
    pub fn is_rooted_by(&self, root: &Rc<Path>) -> bool {
        match &self.value {
            PathEnum::QualifiedPath { qualifier, .. } => {
                *qualifier == *root || qualifier.is_rooted_by(root)
            }
            _ => false,
        }
    }

    /// True if path qualifies an abstract heap block, or another qualified path rooted by an
    /// abstract heap block.
    // The only place that uses this is in handling heap side-effects when a function returns
    pub fn is_rooted_by_abstract_heap_block(&self) -> bool {
        match &self.value {
            PathEnum::QualifiedPath { qualifier, .. } => {
                qualifier.is_rooted_by_abstract_heap_block()
            }
            PathEnum::HeapBlock { .. } => true,
            _ => false,
        }
    }

    /// True if path qualifies a parameter, or another qualified path rooted by a parameter.
    // Used to handle side-effects of function calls
    pub fn is_rooted_by_parameter(&self) -> bool {
        match &self.value {
            PathEnum::QualifiedPath { qualifier, .. } => qualifier.is_rooted_by_parameter(),
            PathEnum::Parameter { .. } => true,
            _ => false,
        }
    }

    /// Returns the length of the path.
    pub fn path_length(&self) -> usize {
        match &self.value {
            PathEnum::QualifiedPath { length, .. } => *length,
            _ => 1,
        }
    }

    /// Creates a path that aliases once or more paths contained inside the value.
    pub fn new_alias(value: Rc<SymbolicValue>) -> Rc<Path> {
        Rc::new(PathEnum::Alias { value }.into())
    }

    /// Creates a path to the target memory of a reference value.
    pub fn new_deref(address_path: Rc<Path>) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Deref);
        Self::new_qualified(address_path, selector)
    }

    /// Creates a path the selects the discriminant of the enum at the given path.
    pub fn new_discriminant(enum_path: Rc<Path>) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Discriminant);
        Self::new_qualified(enum_path, selector)
    }

    /// Creates a path the selects the given field of the struct at the given path.
    pub fn new_field(qualifier: Rc<Path>, field_index: usize) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Field(field_index));
        Self::new_qualified(qualifier, selector)
    }

    /// Creates a path the selects the element at the given index value of the array at the given path.
    pub fn new_index(collection_path: Rc<Path>, index_value: Rc<SymbolicValue>) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Index(index_value));
        Self::new_qualified(collection_path, selector)
    }

    /// Creates a path the selects a slice, [0..count_value], from the value at collection_path.
    pub fn new_slice(collection_path: Rc<Path>, count_value: Rc<SymbolicValue>) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Slice(count_value));
        Self::new_qualified(collection_path, selector)
    }

    pub fn new_static(tcx: TyCtxt<'_>, def_id: DefId) -> Rc<Path> {
        let ty = tcx.type_of(def_id);
        let name = utils::summary_key_str(tcx, def_id);
        Rc::new(
            PathEnum::StaticVariable {
                def_id: Some(def_id),
                summary_cache_key: name,
                expression_type: ExpressionType::from(ty.kind()),
            }
            .into(),
        )
    }

    /// Creates a path to the local variable corresponding to the ordinal.
    pub fn new_local(ordinal: usize, offset: usize) -> Rc<Path> {
        Rc::new(
            PathEnum::LocalVariable {
                ordinal: ordinal + offset,
            }
            .into(),
        )
    }

    /// Creates a path to the local variable corresponding to the ordinal.
    pub fn new_parameter(ordinal: usize, offset: usize) -> Rc<Path> {
        Rc::new(
            PathEnum::Parameter {
                ordinal: ordinal + offset,
            }
            .into(),
        )
    }

    /// Creates a path to the local variable corresponding to the ordinal.
    pub fn new_result() -> Rc<Path> {
        Rc::new(PathEnum::Result.into())
    }

    /// Creates a path to the local variable, parameter or result local, corresponding to the ordinal.
    pub fn new_local_parameter_or_result(
        ordinal: usize,
        offset: usize,
        argument_count: usize,
    ) -> Rc<Path> {
        if ordinal == 0 {
            Self::new_result()
        } else if ordinal <= argument_count {
            Self::new_parameter(ordinal, offset)
        } else {
            Self::new_local(ordinal, offset)
        }
    }

    /// Creates a path the selects the length of the array/slice/string at the given path.
    pub fn new_length(array_path: Rc<Path>) -> Rc<Path> {
        let selector = Rc::new(PathSelector::Field(1));
        Self::new_qualified(array_path, selector)
    }

    /// Creates a path the qualifies the given root path with the given selector.
    pub fn new_qualified(qualifier: Rc<Path>, selector: Rc<PathSelector>) -> Rc<Path> {
        if let PathEnum::Alias { value } = &qualifier.value {
            if value.is_bottom() {
                return qualifier;
            }
        }
        let qualifier_length = qualifier.path_length();

        Rc::new(
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                length: qualifier_length + 1,
            }
            .into(),
        )
    }

    /// Adds any heap blocks found in embedded index values to the given set.
    // Also related to handling side-effects
    // Note that there is also a function with the same name for `Expression`
    pub fn record_heap_blocks(&self, result: &mut HashSet<Rc<SymbolicValue>>) {
        match &self.value {
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } => {
                (**qualifier).record_heap_blocks(result);
                selector.record_heap_blocks(result);
            }
            PathEnum::HeapBlock { value } => {
                if let Expression::HeapBlock { .. } = &value.expression {
                    result.insert(value.clone());
                } else {
                    unreachable!()
                }
            }
            _ => (),
        }
    }

    // TODO: this is only used once in promoted constant, consider removing it
    pub fn get_path_to_field_at_offset_0<'tcx>(
        tcx: TyCtxt<'tcx>,
        // environment: &AbstractDomain,
        path: &Rc<Path>,
        result_rustc_type: Ty<'tcx>,
    ) -> Option<Rc<Path>> {
        trace!(
            "get_path_to_field_at_offset_0 {:?} {:?}",
            path,
            result_rustc_type
        );
        match result_rustc_type.kind() {
            TyKind::Adt(def, substs) => {
                if def.is_enum() {
                    let path0 = Path::new_discriminant(path.clone());
                    return Some(path0);
                }
                let path0 = Path::new_field(path.clone(), 0);
                for v in def.variants.iter() {
                    if let Some(field0) = v.fields.get(0) {
                        let field0_ty = field0.ty(tcx, substs);
                        let result = Self::get_path_to_field_at_offset_0(
                            tcx, // environment,
                            &path0, field0_ty,
                        );
                        if result.is_some() {
                            return result;
                        }
                    }
                }
                None
            }
            TyKind::Tuple(substs) => {
                if let Some(field0_ty) = substs.iter().map(|s| s.expect_ty()).next() {
                    let path0 = Path::new_field(path.clone(), 0);
                    return Self::get_path_to_field_at_offset_0(
                        tcx, // environment,
                        &path0, field0_ty,
                    );
                }
                None
            }
            _ => Some(path.clone()),
        }
    }
}

// Define this trait for defining methods for Rc<Path>
// Still, mainly used to handle function calls
pub trait PathRefinement<DomainType>: Sized
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Refine parameters inside embedded index values with the given arguments.
    fn refine_parameters(
        &self,
        arguments: &[(Rc<Path>, Rc<SymbolicValue>)],
        // fresh: usize,
    ) -> Rc<Path>;

    /// Refine paths that reference other paths.
    /// I.e. when a reference is passed to a function that then returns
    /// or leaks it back to the caller in the qualifier of a path then
    /// we want to dereference the qualifier in order to normalize the path
    /// and not have more than one path for the same location.
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Rc<Path>;

    /// Returns a copy path with the root replaced by new_root.
    fn replace_root(&self, old_root: &Rc<Path>, new_root: Rc<Path>) -> Rc<Path>;
}

impl<DomainType> PathRefinement<DomainType> for Rc<Path>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Refine parameters inside embedded index values with the given arguments.
    fn refine_parameters(
        &self,
        arguments: &[(Rc<Path>, Rc<SymbolicValue>)],
        // fresh: usize,
    ) -> Rc<Path> {
        match &self.value {
            PathEnum::Parameter { ordinal } => {
                if *ordinal > arguments.len() {
                    debug!("Summary refers to a parameter that does not have a matching argument");
                    Path::new_alias(Rc::new(symbolic_value::BOTTOM))
                } else {
                    arguments[*ordinal - 1].0.clone()
                }
            }
            // PathEnum::Result => Path::new_local(fresh),
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } => {
                let refined_qualifier = qualifier.refine_parameters(arguments);
                let refined_selector = selector.refine_parameters(arguments);
                Path::new_qualified(refined_qualifier, refined_selector)
            }
            _ => self.clone(),
        }
    }

    /// Refine paths that reference other paths and canonicalize the refinements.
    /// I.e. when a reference is passed to a function that then returns
    /// or leaks it back to the caller in the qualifier of a path then
    /// we want to dereference the qualifier in order to normalize the path
    /// and not have more than one path for the same location.
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Rc<Path> {
        if let Some(mut val) = environment.value_at(&self) {
            // If the environment has self as a key, then self is canonical, since we should only
            // use canonical paths as keys. The value at the canonical key, however, could just
            // be a reference to another path, which is something that happens during refinement.
            if let Expression::Cast { operand, .. } = &val.expression {
                val = operand.clone();
            }
            return match &val.expression {
                Expression::HeapBlock { .. } => Path::get_as_path(val.refine_paths(environment)),
                Expression::Variable { path, .. } | Expression::Widen { path, .. } => {
                    if let PathEnum::QualifiedPath { selector, .. } = &path.value {
                        if *selector.as_ref() == PathSelector::Deref {
                            // If the path is a deref, it is not just an alias for self, so keep self
                            return self.clone();
                        }
                    }
                    path.clone()
                }
                _ => self.clone(), // self is canonical
            };
        }
        // self is a path that is not a key in the environment. This could be because it is not
        // canonical, which can only be the case if self is a qualified path.
        match &self.value {
            // PathEnum::Offset { value } => Path::get_as_path(value.refine_paths(environment)),
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } => {
                let refined_selector = selector.refine_paths(environment);
                let refined_qualifier = qualifier.refine_paths(environment);

                // The qualifier is now canonical. But in the context of a selector, we
                // might be able to simplify the qualifier by dropping an explicit dereference
                // or an explicit reference.
                if let PathEnum::QualifiedPath {
                    qualifier: base_qualifier,
                    selector: base_selector,
                    ..
                } = &refined_qualifier.value
                {
                    if *base_selector.as_ref() == PathSelector::Deref {
                        // no need for an explicit deref in a qualifier
                        return Path::new_qualified(
                            base_qualifier.clone(),
                            refined_selector.clone(),
                        );
                    }
                }
                if let Some(val) = environment.value_at(&refined_qualifier) {
                    match &val.expression {
                        Expression::Variable { path, .. } => {
                            // if path is a deref we just drop it because it becomes implicit
                            if let PathEnum::QualifiedPath {
                                qualifier,
                                selector: var_path_selector,
                                ..
                            } = &path.value
                            {
                                if let PathSelector::Deref = var_path_selector.as_ref() {
                                    // drop the explicit deref
                                    return Path::new_qualified(
                                        qualifier.clone(),
                                        refined_selector,
                                    );
                                }
                            }
                            return Path::new_qualified(path.clone(), refined_selector);
                        }
                        Expression::Reference(path) => {
                            match refined_selector.as_ref() {
                                PathSelector::Deref => {
                                    // We have a *&path sequence. If path is a is heap block, we
                                    // turn self into path[0]. If not, we drop the sequence and return path.
                                    return if matches!(&path.value, PathEnum::HeapBlock { .. }) {
                                        Path::new_index(path.clone(), Rc::new(0u128.into()))
                                            .refine_paths(environment)
                                    } else {
                                        path.clone()
                                    };
                                }
                                _ => {
                                    // drop the explicit reference
                                    return Path::new_qualified(
                                        path.clone(),
                                        refined_selector.clone(),
                                    );
                                }
                            }
                        }
                        _ => {
                            if val.is_path_alias() {
                                return Path::new_qualified(
                                    Path::new_alias(val.clone()),
                                    refined_selector,
                                );
                            }
                        }
                    }
                    if let Expression::Reference(path) = &val.expression {
                        match refined_selector.as_ref() {
                            PathSelector::Deref => {
                                // if selector is a deref we can just drop the &* sequence
                                return path.clone();
                            }
                            _ => {
                                // drop the explicit reference
                                return Path::new_qualified(path.clone(), refined_selector.clone());
                            }
                        }
                    }
                }
                Path::new_qualified(refined_qualifier, refined_selector)
            }
            _ => {
                self.clone() // Non qualified, non offset paths are already canonical
            }
        }
    }

    /// Returns a copy path with the root replaced by new_root.
    fn replace_root(&self, old_root: &Rc<Path>, new_root: Rc<Path>) -> Rc<Path> {
        match &self.value {
            PathEnum::QualifiedPath {
                qualifier,
                selector,
                ..
            } => {
                let new_qualifier = if *qualifier == *old_root {
                    new_root
                } else {
                    qualifier.replace_root(old_root, new_root)
                };
                Path::new_qualified(new_qualifier, selector.clone())
            }
            _ => new_root,
        }
    }
}

/// The selector denotes a de-referenced item, field, or element, or slice.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PathSelector {
    /// Given a path that denotes a reference, select the thing the reference points to.
    Deref,

    /// The tag used to indicate which case of an enum is used for a particular enum value.
    Discriminant,

    /// Select the struct field with the given index.
    Field(usize),

    /// Select the collection element with the index specified by the abstract value.
    Index(Rc<SymbolicValue>),

    /// Selects slice[0..value] where slice is the qualifier and value the selector parameter.
    Slice(Rc<SymbolicValue>),

    /// These indices are generated by slice patterns. Easiest to explain
    /// by example:
    ///
    /// ```ignore
    /// [X, _, .._, _, _] => { offset: 0, min_length: 4, from_end: false },
    /// [_, X, .._, _, _] => { offset: 1, min_length: 4, from_end: false },
    /// [_, _, .._, X, _] => { offset: 2, min_length: 4, from_end: true },
    /// [_, _, .._, _, X] => { offset: 1, min_length: 4, from_end: true },
    /// ```
    ConstantIndex {
        /// index or -index (in Python terms), depending on from_end
        offset: u64,
        /// The thing being indexed must be at least this long. For arrays this is always the exact length.
        min_length: u64,
        /// counting backwards from end? This is always false when indexing an array.
        from_end: bool,
    },
}

impl Debug for PathSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PathSelector::Deref => f.write_str("deref"),
            PathSelector::Discriminant => f.write_str("discr"),
            PathSelector::Field(index) => index.fmt(f),
            PathSelector::Index(value) => f.write_fmt(format_args!("[{:?}]", value)),
            PathSelector::Slice(value) => f.write_fmt(format_args!("[0..{:?}]", value)),
            PathSelector::ConstantIndex {
                offset,
                min_length,
                from_end,
            } => f.write_fmt(format_args!(
                "[offset: {}, min_length: {}, from_end: {}",
                offset, min_length, from_end
            )),
        }
    }
}

impl PathSelector {
    /// Adds any abstract heap addresses found in embedded index values to the given set.
    pub fn record_heap_blocks(&self, result: &mut HashSet<Rc<SymbolicValue>>) {
        match self {
            PathSelector::Index(value) | PathSelector::Slice(value) => {
                value.record_heap_blocks(result);
            }
            _ => (),
        }
    }
}

pub trait PathSelectorRefinement<DomainType>: Sized
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Refine parameters inside embedded index values with the given arguments.
    fn refine_parameters(&self, arguments: &[(Rc<Path>, Rc<SymbolicValue>)]) -> Self;

    /// Returns a value that is simplified (refined) by replacing values with Variable(path) expressions
    /// with the value at that path (if there is one). If no refinement is possible
    /// the result is simply a clone of this value. This refinement only makes sense
    /// following a call to refine_parameters.
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Self;
}

impl<DomainType> PathSelectorRefinement<DomainType> for Rc<PathSelector>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Refine parameters inside embedded index values with the given arguments.
    fn refine_parameters(
        &self,
        arguments: &[(Rc<Path>, Rc<SymbolicValue>)],
        // fresh: usize,
    ) -> Rc<PathSelector> {
        match self.as_ref() {
            PathSelector::Index(value) => {
                let refined_value = value.refine_parameters(arguments);
                Rc::new(PathSelector::Index(refined_value))
            }
            PathSelector::Slice(value) => {
                let refined_value = value.refine_parameters(arguments);
                Rc::new(PathSelector::Slice(refined_value))
            }
            _ => self.clone(),
        }
    }

    /// Returns a value that is simplified (refined) by replacing values with Variable(path) expressions
    /// with the value at that path (if there is one). If no refinement is possible
    /// the result is simply a clone of this value. This refinement only makes sense
    /// following a call to refine_parameters.
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Rc<PathSelector> {
        match self.as_ref() {
            PathSelector::Index(value) => {
                let refined_value = value.refine_paths(environment);
                Rc::new(PathSelector::Index(refined_value))
            }
            PathSelector::Slice(value) => {
                let refined_value = value.refine_paths(environment);
                Rc::new(PathSelector::Slice(refined_value))
            }
            _ => self.clone(),
        }
    }
}

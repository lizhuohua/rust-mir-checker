// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::expression::{Expression, ExpressionType};
use super::known_names::{KnownNames, KnownNamesCache};
use super::utils;

use az::OverflowingCast;
use rug::Integer;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::subst::SubstsRef;
use rustc_middle::ty::{Ty, TyCtxt};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

/// Represent a compile time constant
#[derive(Clone, Eq, PartialOrd, PartialEq, Hash, Ord)]
pub enum ConstantValue {
    /// The impossible constant. Use this as the result of a partial transfer function.
    Bottom,
    /// The constant that may be any possible value
    Top,
    /// A reference to a function.
    Function(Rc<FunctionReference>),
    /// Integer
    Int(Integer),
}

impl Debug for ConstantValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ConstantValue::Bottom => f.write_str("BOTTOM"),
            ConstantValue::Top => f.write_str("TOP"),
            ConstantValue::Function(func_ref) => f.write_fmt(format_args!(
                "fn {}<{:?}>",
                func_ref.function_name, func_ref.generic_arguments
            )),
            ConstantValue::Int(val) => val.fmt(f),
        }
    }
}

/// Information that identifies a function or generic function instance.
#[derive(Clone, Debug, Eq, PartialOrd, PartialEq, Hash, Ord)]
pub struct FunctionReference {
    /// The crate specific key that is used to identify the function in the current crate.
    /// This is not available for functions returned by calls to functions from other crates,
    /// since the def id the other crates use have no meaning for the current crate.
    pub def_id: Option<DefId>,
    /// A unique identifier for this function reference, derived from the def_id and the
    /// instantiated type of the reference. I.e. every unique instantiation of a generic
    /// function will have a different function_id but the same def_id.
    pub function_id: Option<usize>,
    /// The generic argument types with which the referenced function was instantiated, if generic.
    pub generic_arguments: Vec<ExpressionType>,
    /// Indicates if the function is known to be treated specially by the Rust compiler
    pub known_name: KnownNames,
    /// The name of the function
    pub function_name: Rc<String>,
}

/// Constructors
impl ConstantValue {
    /// Returns a constant value that is a reference to a function
    pub fn for_function<'a, 'tcx, 'compiler>(
        function_id: usize,
        def_id: DefId,
        generic_args: Option<SubstsRef<'tcx>>,
        tcx: TyCtxt<'tcx>,
        known_names_cache: &mut KnownNamesCache,
    ) -> ConstantValue {
        let function_name = utils::summary_key_str(tcx, def_id).to_string();
        let generic_arguments = if let Some(generic_args) = generic_args {
            generic_args.types().map(|t| t.kind().into()).collect()
        } else {
            vec![]
        };
        let known_name = known_names_cache.get(tcx, def_id);
        ConstantValue::Function(Rc::new(FunctionReference {
            def_id: Some(def_id),
            function_id: Some(function_id),
            generic_arguments,
            known_name,
            function_name: Rc::new(function_name.clone()),
        }))
    }

    pub fn is_top(&self) -> bool {
        matches!(self, ConstantValue::Top)
    }

    pub fn is_bottom(&self) -> bool {
        matches!(self, ConstantValue::Bottom)
    }

    pub fn try_get_integer(&self) -> Option<Integer> {
        match self {
            ConstantValue::Int(val) => Some(val.clone()),
            _ => None,
        }
    }
}

impl From<Integer> for ConstantValue {
    fn from(i: Integer) -> ConstantValue {
        ConstantValue::Int(i)
    }
}

impl From<bool> for ConstantValue {
    fn from(b: bool) -> ConstantValue {
        ConstantValue::Int(Integer::from(b))
    }
}

/// Transfer functions
impl ConstantValue {
    /// Returns a constant that is "self + other".
    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() + val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// The Boolean value of this constant, if it is a Boolean constant, otherwise None.
    pub fn as_bool_if_known(&self) -> Option<bool> {
        match &self {
            ConstantValue::Int(val) if *val == 1 => Some(true),
            ConstantValue::Int(val) if *val == 0 => Some(false),
            _ => None,
        }
    }

    // FIXME: Bitwise operations are extremely imprecise

    /// Returns a constant that is "self & other".
    pub fn bit_and(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() & val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "!self" where self is an integer.
    pub fn bit_not(&self, _result_type: ExpressionType) -> Self {
        match self {
            ConstantValue::Int(val) => ConstantValue::Int(!val.clone()),
            ConstantValue::Top => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self | other".
    pub fn bit_or(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() | val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self ^ other".
    pub fn bit_xor(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() ^ val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self as target_type"
    pub fn cast(&self, target_type: &ExpressionType) -> Self {
        match self {
            ConstantValue::Bottom => self.clone(),
            ConstantValue::Int(val) => match target_type {
                ExpressionType::U8 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<u8>::overflowing_cast(val).0,
                )),
                ExpressionType::U16 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<u16>::overflowing_cast(val).0,
                )),
                ExpressionType::U32 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<u32>::overflowing_cast(val).0,
                )),
                ExpressionType::U64 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<u64>::overflowing_cast(val).0,
                )),
                ExpressionType::U128 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<u128>::overflowing_cast(val).0,
                )),
                ExpressionType::Usize => ConstantValue::Int(Integer::from(
                    OverflowingCast::<usize>::overflowing_cast(val).0,
                )),
                ExpressionType::I8 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<i8>::overflowing_cast(val).0,
                )),
                ExpressionType::I16 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<i16>::overflowing_cast(val).0,
                )),
                ExpressionType::I32 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<i32>::overflowing_cast(val).0,
                )),
                ExpressionType::I64 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<i64>::overflowing_cast(val).0,
                )),
                ExpressionType::I128 => ConstantValue::Int(Integer::from(
                    OverflowingCast::<i128>::overflowing_cast(val).0,
                )),
                ExpressionType::Isize => ConstantValue::Int(Integer::from(
                    OverflowingCast::<isize>::overflowing_cast(val).0,
                )),
                _ => self.clone(),
            },
            _ => self.clone(),
        }
    }

    /// Returns a constant that is "self / other".
    pub fn div(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                if *val2 == 0 {
                    ConstantValue::Bottom
                } else {
                    ConstantValue::Int(val1.clone() / val2.clone())
                }
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self == other".
    pub fn equals(&self, other: &Self) -> Self {
        (*self == *other).into()
    }

    /// Returns a constant that is "self >= other".
    pub fn greater_or_equal(&self, other: &Self) -> Self {
        (*self >= *other).into()
    }

    /// Returns a constant that is "self > other".
    pub fn greater_than(&self, other: &Self) -> Self {
        (*self > *other).into()
    }

    /// Returns a constant that is "self <= other".
    pub fn less_or_equal(&self, other: &Self) -> Self {
        (*self <= *other).into()
    }

    /// Returns a constant that is "self < other".
    pub fn less_than(&self, other: &Self) -> Self {
        (*self < *other).into()
    }

    /// Returns a constant that is "self * other".
    pub fn mul(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() * val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "-self".
    pub fn neg(&self) -> Self {
        match self {
            ConstantValue::Int(val) => ConstantValue::Int(-val.clone()),
            ConstantValue::Top => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self != other".
    pub fn not_equals(&self, other: &Self) -> Self {
        (*self != *other).into()
    }

    /// Returns a constant that is "!self" where self is a bool.
    pub fn logical_not(&self) -> Self {
        match self {
            ConstantValue::Int(val) => {
                if *val == 1 {
                    ConstantValue::Int(Integer::from(0))
                } else if *val == 0 {
                    ConstantValue::Int(Integer::from(1))
                } else {
                    ConstantValue::Bottom
                }
            }
            ConstantValue::Top => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self % other".
    pub fn rem(&self, other: &Self) -> Self {
        match (&self, &other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                if *val2 == 0 {
                    ConstantValue::Bottom
                } else {
                    ConstantValue::Int(val1.clone() % val2.clone())
                }
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self << other".
    pub fn shl(&self, other: &Self) -> Self {
        let other_as_u32 = match &other {
            ConstantValue::Int(val) => val.to_u32(),
            _ => None,
        };
        match (&self, other_as_u32) {
            (ConstantValue::Int(val1), Some(val2)) => ConstantValue::Int(val1.clone() << val2),
            (ConstantValue::Top, _) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self >> other".
    pub fn shr(&self, other: &Self) -> Self {
        let other_as_u32 = match &other {
            ConstantValue::Int(val) => val.to_u32(),
            _ => None,
        };
        match (&self, other_as_u32) {
            (ConstantValue::Int(val1), Some(val2)) => ConstantValue::Int(val1.clone() >> val2),
            (ConstantValue::Top, _) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }

    /// Returns a constant that is "self - other".
    pub fn sub(&self, other: &Self) -> Self {
        match (self, other) {
            (ConstantValue::Int(val1), ConstantValue::Int(val2)) => {
                ConstantValue::Int(val1.clone() - val2.clone())
            }
            (ConstantValue::Top, _) | (_, ConstantValue::Top) => ConstantValue::Top,
            _ => ConstantValue::Bottom,
        }
    }
}

/// Keeps track of MIR constants that have already been mapped onto ConstantValue instances.
pub struct ConstantValueCache<'tcx> {
    function_cache: HashMap<(DefId, Ty<'tcx>), ConstantValue>,
    heap_address_counter: usize,
}

impl<'tcx> Debug for ConstantValueCache<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        "ConstantValueCache".fmt(f)
    }
}

impl<'tcx> ConstantValueCache<'tcx> {
    pub fn new() -> ConstantValueCache<'tcx> {
        ConstantValueCache {
            function_cache: HashMap::default(),
            heap_address_counter: 0,
        }
    }

    /// Returns a Expression::HeapBlock with a unique counter value.
    pub fn get_new_heap_block(&mut self) -> Expression {
        let heap_address_counter = self.heap_address_counter;
        self.heap_address_counter = self.heap_address_counter.wrapping_add(1);
        Expression::HeapBlock {
            abstract_address: heap_address_counter,
            // is_zeroed,
        }
    }

    /// Given the MIR DefId of a function return the unique (cached) ConstantValue that corresponds
    /// to the function identified by that DefId.
    pub fn get_function_constant_for<'a, 'compiler>(
        &mut self,
        def_id: DefId,
        ty: Ty<'tcx>,
        generic_args: Option<SubstsRef<'tcx>>,
        tcx: TyCtxt<'tcx>,
        known_names_cache: &mut KnownNamesCache,
    ) -> &ConstantValue {
        let function_id = self.function_cache.len();
        self.function_cache.entry((def_id, ty)).or_insert_with(|| {
            ConstantValue::for_function(function_id, def_id, generic_args, tcx, known_names_cache)
        })
    }

    /// Resets the heap block counter to 0.
    /// Do this for every function body to ensure that its analysis is not dependent on what
    /// happened elsewhere. Also remember to relocate heap addresses from summaries of other
    /// functions when transferring callee state to the caller's state.
    pub fn reset_heap_counter(&mut self) {
        self.heap_address_counter = 0;
    }
}

impl<'tcx> Default for ConstantValueCache<'tcx> {
    fn default() -> Self {
        Self::new()
    }
}

// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::constant_value::ConstantValue;
use super::path::Path;
use super::symbolic_value::SymbolicValue;
use rug::Integer;

use rustc_ast::ast;
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

/// Closely based on the expressions found in MIR.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Expression {
    /// A symbol indicating that a path has been dropped by `TerminatorKind::Drop`
    Drop(Rc<Path>),

    /// An integer value that stores in the numerical abstract domain
    Numerical(Rc<Path>),

    /// An expression that represents any possible value
    Top,

    /// An expression that represents an impossible value, such as the value returned by function
    /// that always panics.
    Bottom,

    /// An expression that is true if both left and right are true. &&
    And {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is the operand cast to the target_type. as
    Cast {
        // The value of the operand.
        operand: Rc<SymbolicValue>,
        // The type the operand is being cast to.
        target_type: ExpressionType,
    },

    /// An expression that is a compile time constant value, such as a numeric literal or a function.
    CompileTimeConstant(ConstantValue),

    /// An expression that is true if left and right are equal. ==
    Equals {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if left is greater than or equal to right. >=
    GreaterOrEqual {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if left is greater than right. >
    GreaterThan {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that represents a block of memory allocated from the heap.
    /// The value of expression is an ordinal used to distinguish this allocation from
    /// other allocations. Because this is static analysis, a given allocation site will
    /// always result in the same ordinal. The implication of this is that there will be
    /// some loss of precision when heap blocks are allocated inside loops.
    HeapBlock {
        // A unique ordinal that distinguishes this allocation from other allocations.
        // Not an actual memory address.
        abstract_address: usize,
        // // True if the allocator zeroed out this heap memory block.
        // is_zeroed: bool,
    },

    /// Either left or right without a condition to tell them apart.
    /// This can happen when there is loss of precision because of a loop fixed point computation.
    /// For instance inside a loop body after the second fixed point iteration, a counter may have
    /// either its initial value or the value computed in the first loop body iteration and we
    /// don't have a way to tell which value it is.
    Join {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if left is less than or equal to right. <=
    LessOrEqual {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if left is less than right. <
    LessThan {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if the operand is false. ! bool
    LogicalNot { operand: Rc<SymbolicValue> },

    /// An expression that is true if left and right are not equal. !=
    Ne {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// An expression that is true if either one of left or right are true. ||
    Or {
        // The value of the left operand.
        left: Rc<SymbolicValue>,
        // The value of the right operand.
        right: Rc<SymbolicValue>,
    },

    /// The corresponding concrete value is the runtime address of location identified by the path.
    Reference(Rc<Path>),

    /// The unknown value of a place in memory.
    /// This is distinct from Top in that we known something: the place and the type.
    /// This is a useful distinction because it allows us to simplify some expressions
    /// like x == x. The type is needed to prevent this particular optimization if
    /// the variable is a floating point number that could be NaN.
    Variable {
        path: Rc<Path>,
        var_type: ExpressionType,
    },

    /// The partly known value of a place in memory.
    /// The value in operand will be the join of several expressions that all reference
    /// the path of this value. This models a variable that is assigned to from inside a loop
    /// body.
    Widen {
        /// The path of the location where an indeterminate number of flows join together.
        path: Rc<Path>,
        /// The join of some of the flows to come together at this path.
        /// The first few iterations do joins. Once widening happens, further iterations
        /// all result in the same widened value.
        operand: Rc<SymbolicValue>,
    },
}

impl Debug for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expression::Drop(path) => f.write_fmt(format_args!("Drop({:?})", path)),
            Expression::Numerical(path) => f.write_fmt(format_args!("Numerical({:?})", path)),
            Expression::Top => f.write_str("TOP"),
            Expression::Bottom => f.write_str("BOTTOM"),
            Expression::And { left, right } => {
                f.write_fmt(format_args!("({:?}) && ({:?})", left, right))
            }
            Expression::Cast {
                operand,
                target_type,
            } => f.write_fmt(format_args!("({:?}) as {:?}", operand, target_type)),
            Expression::CompileTimeConstant(c) => c.fmt(f),
            Expression::Equals { left, right } => {
                f.write_fmt(format_args!("({:?}) == ({:?})", left, right))
            }
            Expression::GreaterOrEqual { left, right } => {
                f.write_fmt(format_args!("({:?}) >= ({:?})", left, right))
            }
            Expression::GreaterThan { left, right } => {
                f.write_fmt(format_args!("({:?}) > ({:?})", left, right))
            }
            Expression::HeapBlock {
                abstract_address: address,
                // is_zeroed,
            } => f.write_fmt(format_args!(
                "heap_{}",
                // if *is_zeroed { "zeroed_" } else { "" },
                *address,
            )),
            Expression::LessOrEqual { left, right } => {
                f.write_fmt(format_args!("({:?}) <= ({:?})", left, right))
            }
            Expression::LessThan { left, right } => {
                f.write_fmt(format_args!("({:?}) < ({:?})", left, right))
            }
            Expression::LogicalNot { operand } => f.write_fmt(format_args!("!({:?})", operand)),
            Expression::Ne { left, right } => {
                f.write_fmt(format_args!("({:?}) != ({:?})", left, right))
            }
            Expression::Or { left, right } => {
                f.write_fmt(format_args!("({:?}) || ({:?})", left, right))
            }
            Expression::Reference(path) => f.write_fmt(format_args!("&({:?})", path)),
            Expression::Variable { path, var_type } => {
                f.write_fmt(format_args!("{:?}: {:?}", path, var_type))
            }
            Expression::Widen { path, operand } => {
                if operand.expression_size > 100 {
                    f.write_fmt(format_args!("widen(..) at {:?}", path))
                } else {
                    f.write_fmt(format_args!("widen({:?}) at {:?}", operand, path))
                }
            }
            Expression::Join { left, right } => {
                f.write_fmt(format_args!("({:?}) join ({:?})", left, right))
            }
        }
    }
}

impl Expression {
    /// Returns the type of value the expression should result in, if well formed.
    /// (both operands are of the same type for binary operators, conditional branches match).
    pub fn infer_type(&self) -> ExpressionType {
        use self::ExpressionType::*;
        match self {
            Expression::Drop(..) => NonPrimitive,
            Expression::Top => NonPrimitive,
            Expression::Bottom => NonPrimitive,
            Expression::And { .. } => Bool,
            Expression::HeapBlock { .. } => NonPrimitive,
            Expression::Cast { target_type, .. } => target_type.clone(),
            Expression::CompileTimeConstant(c) => c.into(),
            Expression::Equals { .. } => Bool,
            Expression::GreaterOrEqual { .. } => Bool,
            Expression::GreaterThan { .. } => Bool,
            Expression::Join { left, .. } => left.expression.infer_type(),
            Expression::LessOrEqual { .. } => Bool,
            Expression::LessThan { .. } => Bool,
            Expression::LogicalNot { .. } => Bool,
            Expression::Ne { .. } => Bool,
            Expression::Or { .. } => Bool,
            Expression::Reference(_) => Reference,
            Expression::Variable { var_type, .. } => var_type.clone(),
            Expression::Widen { operand, .. } => operand.expression.infer_type(),
            // TODO: simply regarding numerical values as `i128` is not precise
            Expression::Numerical(..) => I128,
        }
    }

    /// Determines if the given expression is the compile time constant 1u128.
    pub fn is_one(&self) -> bool {
        if let Expression::CompileTimeConstant(ConstantValue::Int(val)) = self {
            return *val == 1;
        }
        false
    }

    /// Determines if the given expression is the compile time constant 0u128.
    pub fn is_zero(&self) -> bool {
        if let Expression::CompileTimeConstant(ConstantValue::Int(val)) = self {
            return *val == 0;
        }
        false
    }

    /// Adds any heap blocks found in the associated expression to the given set.
    pub fn record_heap_blocks(&self, result: &mut HashSet<Rc<SymbolicValue>>) {
        match &self {
            Expression::And { left, right }
            | Expression::Equals { left, right }
            | Expression::GreaterOrEqual { left, right }
            | Expression::GreaterThan { left, right }
            | Expression::LessOrEqual { left, right }
            | Expression::LessThan { left, right }
            | Expression::Ne { left, right }
            | Expression::Or { left, right } => {
                left.expression.record_heap_blocks(result);
                right.expression.record_heap_blocks(result);
            }
            Expression::HeapBlock { .. } => {
                result.insert(SymbolicValue::make_from(self.clone(), 1));
            }
            Expression::Reference(path) => path.record_heap_blocks(result),
            Expression::Variable { path, .. } => path.record_heap_blocks(result),
            _ => (),
        }
    }
}

/// The type of a place in memory, as understood by MIR.
/// For now, we are only really interested to distinguish between
/// floating point values and other values, because NaN != NaN.
/// In the future the other distinctions may be helpful to SMT solvers.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ExpressionType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    NonPrimitive,
    Reference,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl From<&ConstantValue> for ExpressionType {
    fn from(cv: &ConstantValue) -> ExpressionType {
        use self::ExpressionType::*;
        match cv {
            ConstantValue::Bottom => NonPrimitive,
            ConstantValue::Function { .. } => Reference,
            ConstantValue::Int(..) => I128,
            ConstantValue::Top => Reference,
        }
    }
}

impl<'a> From<&TyKind<'a>> for ExpressionType {
    fn from(ty_kind: &TyKind<'a>) -> ExpressionType {
        match ty_kind {
            TyKind::Bool => ExpressionType::Bool,
            TyKind::Int(ast::IntTy::Isize) => ExpressionType::Isize,
            TyKind::Int(ast::IntTy::I8) => ExpressionType::I8,
            TyKind::Int(ast::IntTy::I16) => ExpressionType::I16,
            TyKind::Int(ast::IntTy::I32) => ExpressionType::I32,
            TyKind::Int(ast::IntTy::I64) => ExpressionType::I64,
            TyKind::Int(ast::IntTy::I128) => ExpressionType::I128,
            TyKind::Uint(ast::UintTy::Usize) => ExpressionType::Usize,
            TyKind::Uint(ast::UintTy::U8) => ExpressionType::U8,
            TyKind::Uint(ast::UintTy::U16) => ExpressionType::U16,
            TyKind::Uint(ast::UintTy::U32) => ExpressionType::U32,
            TyKind::Uint(ast::UintTy::U64) => ExpressionType::U64,
            TyKind::Uint(ast::UintTy::U128) => ExpressionType::U128,
            TyKind::Closure(..)
            | TyKind::Dynamic(..)
            | TyKind::Foreign(..)
            | TyKind::FnDef(..)
            | TyKind::FnPtr(..)
            | TyKind::Generator(..)
            | TyKind::GeneratorWitness(..)
            | TyKind::RawPtr(..)
            | TyKind::Ref(..)
            | TyKind::Slice(..)
            | TyKind::Str => ExpressionType::Reference,
            _ => ExpressionType::NonPrimitive,
        }
    }
}

impl ExpressionType {
    pub fn as_rustc_type<'a>(&self, tcx: TyCtxt<'a>) -> Ty<'a> {
        use self::ExpressionType::*;
        match self {
            Bool => tcx.types.bool,
            I8 => tcx.types.i8,
            I16 => tcx.types.i16,
            I32 => tcx.types.i32,
            I64 => tcx.types.i64,
            I128 => tcx.types.i128,
            Isize => tcx.types.isize,
            U8 => tcx.types.u8,
            U16 => tcx.types.u16,
            U32 => tcx.types.u32,
            U64 => tcx.types.u64,
            U128 => tcx.types.u128,
            Usize => tcx.types.usize,
            Reference => tcx.mk_ty(TyKind::Str),
            NonPrimitive => tcx.types.trait_object_dummy_self,
        }
    }

    /// Returns true if this type is one of the integer types.
    pub fn is_integer(&self) -> bool {
        use self::ExpressionType::*;
        match self {
            I8 | I16 | I32 | I64 | I128 | Isize | U8 | U16 | U32 | U64 | U128 | Usize | Bool => {
                true
            }
            _ => false,
        }
    }

    /// Returns true if this type is not a primitive type. References are not regarded as
    /// primitives for this purpose.
    pub fn is_primitive(&self) -> bool {
        use self::ExpressionType::*;
        !matches!(self, NonPrimitive | Reference)
    }

    /// Returns true if this type is one of the signed integer types.
    pub fn is_signed_integer(&self) -> bool {
        use self::ExpressionType::*;
        matches!(self, I8 | I16 | I32 | I64 | I128 | Isize)
    }

    /// Returns true if this type is one of the unsigned integer types.
    pub fn is_unsigned_integer(&self) -> bool {
        use self::ExpressionType::*;
        matches!(self, U8 | U16 | U32 | U64 | U128 | Usize)
    }

    /// Returns the number of bits used to represent the given type, if primitive.
    /// For non primitive types the result is just 0.
    pub fn bit_length(&self) -> u8 {
        use self::ExpressionType::*;
        match self {
            Bool => 1,
            I8 => 8,
            I16 => 16,
            I32 => 32,
            I64 => 64,
            I128 => 128,
            Isize => 64,
            U8 => 8,
            U16 => 16,
            U32 => 32,
            U64 => 64,
            U128 => 128,
            Usize => 64,
            Reference => 128,
            NonPrimitive => 128,
        }
    }

    /// Returns the maximum value for this type, as a ConstantValue element.
    /// If the type is not a primitive value, the result is Bottom.
    pub fn max_value(&self) -> ConstantValue {
        use self::ExpressionType::*;
        match self {
            Bool => ConstantValue::Int(Integer::from(1)),
            I8 => ConstantValue::Int(Integer::from(std::i8::MAX)),
            I16 => ConstantValue::Int(Integer::from(std::i16::MAX)),
            I32 => ConstantValue::Int(Integer::from(std::i32::MAX)),
            I64 => ConstantValue::Int(Integer::from(std::i64::MAX)),
            I128 => ConstantValue::Int(Integer::from(std::i128::MAX)),
            Isize => ConstantValue::Int(Integer::from(std::isize::MAX)),
            U8 => ConstantValue::Int(Integer::from(std::u8::MAX)),
            U16 => ConstantValue::Int(Integer::from(std::u16::MAX)),
            U32 => ConstantValue::Int(Integer::from(std::u32::MAX)),
            U64 => ConstantValue::Int(Integer::from(std::u64::MAX)),
            U128 => ConstantValue::Int(Integer::from(std::u128::MAX)),
            Usize => ConstantValue::Int(Integer::from(std::usize::MAX)),
            _ => ConstantValue::Bottom,
        }
    }

    pub fn max_value_int(&self) -> Integer {
        use self::ExpressionType::*;
        match self {
            Bool => Integer::from(1),
            I8 => Integer::from(std::i8::MAX),
            I16 => Integer::from(std::i16::MAX),
            I32 => Integer::from(std::i32::MAX),
            I64 => Integer::from(std::i64::MAX),
            I128 => Integer::from(std::i128::MAX),
            Isize => Integer::from(std::isize::MAX),
            U8 => Integer::from(std::u8::MAX),
            U16 => Integer::from(std::u16::MAX),
            U32 => Integer::from(std::u32::MAX),
            U64 => Integer::from(std::u64::MAX),
            U128 => Integer::from(std::u128::MAX),
            Usize => Integer::from(std::usize::MAX),
            _ => unreachable!(),
        }
    }

    /// Returns the minimum value for this type, as a ConstantValue element.
    /// If the type is not a primitive value, the result is Bottom.
    pub fn min_value(&self) -> ConstantValue {
        use self::ExpressionType::*;
        match self {
            Bool => ConstantValue::Int(Integer::from(0)),
            I8 => ConstantValue::Int(Integer::from(std::i8::MIN)),
            I16 => ConstantValue::Int(Integer::from(std::i16::MIN)),
            I32 => ConstantValue::Int(Integer::from(std::i32::MIN)),
            I64 => ConstantValue::Int(Integer::from(std::i64::MIN)),
            I128 => ConstantValue::Int(Integer::from(std::i128::MIN)),
            Isize => ConstantValue::Int(Integer::from(std::isize::MIN)),
            U8 => ConstantValue::Int(Integer::from(std::u8::MIN)),
            U16 => ConstantValue::Int(Integer::from(std::u16::MIN)),
            U32 => ConstantValue::Int(Integer::from(std::u32::MIN)),
            U64 => ConstantValue::Int(Integer::from(std::u64::MIN)),
            U128 => ConstantValue::Int(Integer::from(std::u128::MIN)),
            Usize => ConstantValue::Int(Integer::from(std::usize::MIN)),
            _ => ConstantValue::Bottom,
        }
    }

    pub fn min_value_int(&self) -> Integer {
        use self::ExpressionType::*;
        match self {
            Bool => Integer::from(0),
            I8 => Integer::from(std::i8::MIN),
            I16 => Integer::from(std::i16::MIN),
            I32 => Integer::from(std::i32::MIN),
            I64 => Integer::from(std::i64::MIN),
            I128 => Integer::from(std::i128::MIN),
            Isize => Integer::from(std::isize::MIN),
            U8 => Integer::from(std::u8::MIN),
            U16 => Integer::from(std::u16::MIN),
            U32 => Integer::from(std::u32::MIN),
            U64 => Integer::from(std::u64::MIN),
            U128 => Integer::from(std::u128::MIN),
            Usize => Integer::from(std::usize::MIN),
            _ => unreachable!(),
        }
    }

    /// Returns the maximum value for this type, plus one, as an abstract value.
    /// If the type is not a primitive integer value, the result is Bottom.
    pub fn modulo_value(&self) -> Rc<SymbolicValue> {
        use self::ExpressionType::*;
        match self {
            U8 => Rc::new(ConstantValue::Int(Integer::from(std::u8::MAX) + 1).into()),
            U16 => Rc::new(ConstantValue::Int(Integer::from(std::u16::MAX) + 1).into()),
            U32 => Rc::new(ConstantValue::Int(Integer::from(std::u32::MAX) + 1).into()),
            U64 => Rc::new(ConstantValue::Int(Integer::from(std::u64::MAX) + 1).into()),
            Usize => Rc::new(ConstantValue::Int(Integer::from(std::usize::MAX) + 1).into()),
            _ => Rc::new(ConstantValue::Bottom.into()),
        }
    }
}

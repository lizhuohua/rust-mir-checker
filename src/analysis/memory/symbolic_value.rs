// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::constant_value::ConstantValue;
use super::expression::{Expression, ExpressionType};
use super::k_limits;
use super::path::PathRefinement;
use super::path::{Path, PathEnum, PathSelector};
use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use rug::Integer;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;

/// Represent a symbolic value. This is mainly used as our memory model
#[derive(Clone, Eq, Ord, PartialOrd)]
pub struct SymbolicValue {
    pub expression: Expression,
    pub expression_size: u64,
}

impl Debug for SymbolicValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.expression.fmt(f)
    }
}

impl Hash for SymbolicValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.expression.hash(state);
    }
}

impl PartialEq for SymbolicValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.expression, &other.expression) {
            // Assume widened values are equal
            (Expression::Widen { path: p1, .. }, Expression::Widen { path: p2, .. }) => p1.eq(p2),
            (e1, e2) => e1.eq(e2),
        }
    }
}

/// An abstract domain element that all represent the impossible concrete value.
/// I.e. the corresponding set of possible concrete values is empty.
pub const BOTTOM: SymbolicValue = SymbolicValue {
    expression: Expression::Bottom,
    expression_size: 1,
};

/// An abstract domain element that all represents all possible concrete values.
pub const TOP: SymbolicValue = SymbolicValue {
    expression: Expression::Top,
    expression_size: 1,
};

impl From<bool> for SymbolicValue {
    fn from(b: bool) -> SymbolicValue {
        if b {
            SymbolicValue {
                expression: Expression::CompileTimeConstant(ConstantValue::Int(Integer::from(1))),
                expression_size: 1,
            }
        } else {
            SymbolicValue {
                expression: Expression::CompileTimeConstant(ConstantValue::Int(Integer::from(0))),
                expression_size: 1,
            }
        }
    }
}

impl From<ConstantValue> for SymbolicValue {
    fn from(cv: ConstantValue) -> SymbolicValue {
        if let ConstantValue::Bottom = &cv {
            BOTTOM
        } else {
            SymbolicValue {
                expression: Expression::CompileTimeConstant(cv),
                expression_size: 1,
            }
        }
    }
}

impl From<u128> for SymbolicValue {
    fn from(cv: u128) -> SymbolicValue {
        SymbolicValue {
            expression: Expression::CompileTimeConstant(ConstantValue::Int(Integer::from(cv))),
            expression_size: 1,
        }
    }
}

impl SymbolicValue {
    pub fn new_true() -> Self {
        SymbolicValue {
            expression: Expression::CompileTimeConstant(ConstantValue::Int(Integer::from(1))),
            expression_size: 1,
        }
    }

    pub fn new_false() -> Self {
        SymbolicValue {
            expression: Expression::CompileTimeConstant(ConstantValue::Int(Integer::from(0))),
            expression_size: 1,
        }
    }

    /// Creates an abstract value from a binary expression and keeps track of the size.
    fn make_binary(
        left: Rc<SymbolicValue>,
        right: Rc<SymbolicValue>,
        operation: fn(Rc<SymbolicValue>, Rc<SymbolicValue>) -> Expression,
    ) -> Rc<SymbolicValue> {
        if left.is_top() || left.is_bottom() {
            return left;
        }
        if right.is_top() || right.is_bottom() {
            return right;
        }
        let expression_size = left.expression_size.saturating_add(right.expression_size);
        Self::make_from(operation(left, right), expression_size)
    }

    /// Creates an abstract value from a typed unary expression and keeps track of the size.
    fn make_typed_unary(
        operand: Rc<SymbolicValue>,
        result_type: ExpressionType,
        operation: fn(Rc<SymbolicValue>, ExpressionType) -> Expression,
    ) -> Rc<SymbolicValue> {
        let expression_size = operand.expression_size.saturating_add(1);
        Self::make_from(operation(operand, result_type), expression_size)
    }

    /// Creates an abstract value from a unary expression and keeps track of the size.
    fn make_unary(
        operand: Rc<SymbolicValue>,
        operation: fn(Rc<SymbolicValue>) -> Expression,
    ) -> Rc<SymbolicValue> {
        let expression_size = operand.expression_size.saturating_add(1);
        Self::make_from(operation(operand), expression_size)
    }

    /// Creates an abstract value from the given expression and size.
    /// Initializes the optional domains to None.
    pub fn make_from(expression: Expression, expression_size: u64) -> Rc<SymbolicValue> {
        if expression_size > k_limits::MAX_EXPRESSION_SIZE {
            // If the expression gets too large, refining it gets expensive and composing it
            // into other expressions leads to exponential growth. We therefore need to abstract
            // (go up in the lattice). We do that by making the expression a typed variable and
            // by eagerly computing and caching any other domains, such as the interval domain.
            let var_type = expression.infer_type();
            // FIXME
            let _val = Rc::new(SymbolicValue {
                expression,
                expression_size,
            });
            Rc::new(SymbolicValue {
                expression: Expression::Variable {
                    path: Path::new_alias(TOP.into()),
                    var_type,
                },
                expression_size: 1,
            })
        } else {
            Rc::new(SymbolicValue {
                expression,
                expression_size,
            })
        }
    }

    /// Creates an abstract value that is a reference to the memory named by the given path.
    pub fn make_reference(path: Rc<Path>) -> Rc<SymbolicValue> {
        let path_length = path.path_length() as u64;
        SymbolicValue::make_from(Expression::Reference(path), path_length)
    }

    /// Creates an abstract value about which nothing is known other than its type.
    pub fn make_typed_unknown(var_type: ExpressionType) -> Rc<SymbolicValue> {
        SymbolicValue::make_from(
            Expression::Variable {
                path: Path::new_alias(TOP.into()),
                var_type,
            },
            1,
        )
    }
}

/// Some methods that a symbolic value has
/// Define a trait in order to define these methods for type `Rc<SymbolicValue>`
pub trait SymbolicValueTrait: Sized {
    fn and(&self, other: Self) -> Self;
    fn as_bool_if_known(&self) -> Option<bool>;
    fn as_int_if_known(&self) -> Option<Integer>;
    fn cast(&self, target_type: ExpressionType) -> Self;
    fn dereference(&self, target_type: ExpressionType) -> Self;
    fn equals(&self, other: Self) -> Self;
    fn greater_or_equal(&self, other: Self) -> Self;
    fn greater_than(&self, other: Self) -> Self;
    fn implies(&self, other: &Self) -> bool;
    fn implies_not(&self, other: &Self) -> bool;
    fn inverse_implies(&self, other: &Rc<SymbolicValue>) -> bool;
    fn inverse_implies_not(&self, other: &Rc<SymbolicValue>) -> bool;
    fn is_bottom(&self) -> bool;
    fn is_compile_time_constant(&self) -> bool;
    // fn is_contained_in_zeroed_heap_block(&self) -> bool;
    fn is_path_alias(&self) -> bool;
    fn is_top(&self) -> bool;
    fn is_widened(&self) -> bool;
    fn join(&self, other: Self) -> Self;
    fn less_or_equal(&self, other: Self) -> Self;
    fn less_than(&self, other: Self) -> Self;
    fn not_equals(&self, other: Self) -> Self;
    fn logical_not(&self) -> Self;
    fn or(&self, other: Self) -> Self;
    fn record_heap_blocks(&self, result: &mut HashSet<Rc<SymbolicValue>>);
    fn subset(&self, other: &Self) -> bool;
    fn refine_with(&self, path_condition: &Self, depth: usize) -> Self;
    fn widen(&self, path: &Rc<Path>) -> Self;
    fn depend_on_path_value(&self, path: &Rc<Path>, value: &Rc<SymbolicValue>) -> bool;
}

/// Two methods that are used to refine a symbolic value
/// Define a trait in order to define these methods for type `Rc<SymbolicValue>`
pub trait SymbolicValueRefinement<DomainType>: Sized
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Self;
    fn refine_parameters(&self, arguments: &[(Rc<Path>, Rc<SymbolicValue>)]) -> Self;
}

impl<DomainType> SymbolicValueRefinement<DomainType> for Rc<SymbolicValue>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    /// Replaces occurrences of Expression::Variable(path) with the value at that path
    /// in the given environment (if there is such a value).
    fn refine_paths(&self, environment: &AbstractDomain<DomainType>) -> Rc<SymbolicValue> {
        match &self.expression {
            Expression::Drop(..) => self.clone(),
            Expression::Numerical(..) => self.clone(),
            Expression::Bottom | Expression::Top => self.clone(),
            Expression::And { left, right } => left
                .refine_paths(environment)
                .and(right.refine_paths(environment)),
            Expression::CompileTimeConstant(..) => self.clone(),
            Expression::Cast {
                operand,
                target_type,
            } => operand.refine_paths(environment).cast(target_type.clone()),
            Expression::Equals { left, right } => left
                .refine_paths(environment)
                .equals(right.refine_paths(environment)),
            Expression::GreaterOrEqual { left, right } => left
                .refine_paths(environment)
                .greater_or_equal(right.refine_paths(environment)),
            Expression::GreaterThan { left, right } => left
                .refine_paths(environment)
                .greater_than(right.refine_paths(environment)),
            Expression::HeapBlock { .. } => self.clone(),
            Expression::Join { left, right } => left
                .refine_paths(environment)
                .join(right.refine_paths(environment)),
            Expression::LessOrEqual { left, right } => left
                .refine_paths(environment)
                .less_or_equal(right.refine_paths(environment)),
            Expression::LessThan { left, right } => left
                .refine_paths(environment)
                .less_than(right.refine_paths(environment)),
            Expression::Ne { left, right } => left
                .refine_paths(environment)
                .not_equals(right.refine_paths(environment)),
            Expression::LogicalNot { operand } => operand.refine_paths(environment).logical_not(),
            Expression::Or { left, right } => left
                .refine_paths(environment)
                .or(right.refine_paths(environment)),
            Expression::Reference(path) => {
                let refined_path = path.refine_paths(environment);
                SymbolicValue::make_reference(refined_path)
            }
            Expression::Variable { path, var_type } => {
                if let Some(val) = environment.value_at(&path) {
                    val
                } else {
                    let refined_path = path.refine_paths(environment);
                    if let PathEnum::Alias { value } = &refined_path.value {
                        value.clone()
                    } else if let Some(val) = environment.value_at(&refined_path) {
                        val
                    } else if refined_path == *path {
                        self.clone()
                    } else {
                        SymbolicValue::make_from(
                            Expression::Variable {
                                path: refined_path,
                                var_type: var_type.clone(),
                            },
                            1,
                        )
                    }
                }
            }
            Expression::Widen { path, operand, .. } => operand
                .refine_paths(environment)
                .widen(&path.refine_paths(environment)),
        }
    }

    /// Returns a value that is simplified (refined) by replacing parameter values
    /// with their corresponding argument values. If no refinement is possible
    /// the result is simply a clone of this value.
    fn refine_parameters(
        &self,
        arguments: &[(Rc<Path>, Rc<SymbolicValue>)],
        // fresh: usize,
    ) -> Rc<SymbolicValue> {
        match &self.expression {
            Expression::Drop(..) => self.clone(),
            Expression::Numerical(..) => self.clone(),
            Expression::Bottom | Expression::Top => self.clone(),
            Expression::And { left, right } => left
                .refine_parameters(arguments)
                .and(right.refine_parameters(arguments)),
            Expression::CompileTimeConstant(..) => self.clone(),
            Expression::Cast {
                operand,
                target_type,
            } => operand
                .refine_parameters(arguments)
                .cast(target_type.clone()),
            Expression::Equals { left, right } => left
                .refine_parameters(arguments)
                .equals(right.refine_parameters(arguments)),
            Expression::GreaterOrEqual { left, right } => left
                .refine_parameters(arguments)
                .greater_or_equal(right.refine_parameters(arguments)),
            Expression::GreaterThan { left, right } => left
                .refine_parameters(arguments)
                .greater_than(right.refine_parameters(arguments)),
            Expression::HeapBlock { .. } => self.clone(),
            Expression::Join { left, right } => left
                .refine_parameters(arguments)
                .join(right.refine_parameters(arguments)),
            Expression::LessOrEqual { left, right } => left
                .refine_parameters(arguments)
                .less_or_equal(right.refine_parameters(arguments)),
            Expression::LessThan { left, right } => left
                .refine_parameters(arguments)
                .less_than(right.refine_parameters(arguments)),
            Expression::LogicalNot { operand } => {
                operand.refine_parameters(arguments).logical_not()
            }
            Expression::Ne { left, right } => left
                .refine_parameters(arguments)
                .not_equals(right.refine_parameters(arguments)),
            Expression::Or { left, right } => left
                .refine_parameters(arguments)
                .or(right.refine_parameters(arguments)),
            Expression::Reference(path) => {
                // if the path is a parameter, the reference is an artifact of its type
                // and needs to be removed in the call context
                match &path.value {
                    PathEnum::Parameter { ordinal } => arguments[*ordinal - 1].1.clone(),
                    _ => {
                        let refined_path = path.refine_parameters(arguments);
                        SymbolicValue::make_reference(refined_path)
                    }
                }
            }
            Expression::Variable { path, var_type } => {
                let refined_path = path.refine_parameters(arguments);
                if let PathEnum::Alias { value } = &refined_path.value {
                    value.clone()
                } else {
                    SymbolicValue::make_from(
                        Expression::Variable {
                            path: refined_path,
                            var_type: var_type.clone(),
                        },
                        1,
                    )
                }
            }
            Expression::Widen { path, operand, .. } => operand
                .refine_parameters(arguments)
                .widen(&path.refine_parameters(arguments)),
        }
    }
}

impl SymbolicValueTrait for Rc<SymbolicValue> {
    /// Test whether self depends on the value of path
    fn depend_on_path_value(&self, path: &Rc<Path>, value: &Rc<SymbolicValue>) -> bool {
        if self == value {
            true
        } else {
            match &self.expression {
                Expression::Numerical(p)
                | Expression::Reference(p)
                | Expression::Variable { path: p, .. } => path == p || p.is_rooted_by(path),
                Expression::And { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::Cast { operand, .. } => operand.depend_on_path_value(path, value),
                Expression::Equals { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::GreaterOrEqual { left, right } => {
                    debug!("In depend_on_path_value, greater_or_equal, {:?}", path);
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::GreaterThan { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::Join { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::LessOrEqual { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::LessThan { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::LogicalNot { operand } => operand.depend_on_path_value(path, value),
                Expression::Ne { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::Or { left, right } => {
                    left.depend_on_path_value(path, value)
                        || right.depend_on_path_value(path, value)
                }
                Expression::Widen { operand, .. } => operand.depend_on_path_value(path, value),
                _ => false,
            }
        }
    }
    /// Returns an element that is "self && other".
    fn and(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        let self_bool = self.as_bool_if_known();
        if let Some(false) = self_bool {
            // [false && other] -> false
            return Rc::new(SymbolicValue::new_false());
        };
        let other_bool = other.as_bool_if_known();
        if let Some(false) = other_bool {
            // [self && false] -> false
            return Rc::new(SymbolicValue::new_false());
        };
        if self_bool.unwrap_or(false) {
            if other_bool.unwrap_or(false) {
                // [true && true] -> true
                Rc::new(SymbolicValue::new_true())
            } else {
                // [true && other] -> other
                other
            }
        } else if other_bool.unwrap_or(false) || self.is_bottom() {
            // [self && true] -> self
            // [BOTTOM && other] -> BOTTOM
            self.clone()
        } else if other.is_bottom() {
            // [self && BOTTOM] -> BOTTOM
            other
        } else {
            match &self.expression {
                Expression::And { left: x, right: y } => {
                    // [(x && y) && x] -> x && y
                    // [(x && y) && y] -> x && y
                    if *x == other || *y == other {
                        return self.clone();
                    }
                }
                Expression::LogicalNot { operand } if *operand == other => {
                    // [!x && x] -> false
                    return Rc::new(SymbolicValue::new_false());
                }
                Expression::Or { left: x, right: y } => {
                    // [(x || y) && x] -> x
                    // [(x || y) && y] -> y
                    if *x == other || *y == other {
                        return other;
                    }
                    if let Expression::LogicalNot { operand } = &other.expression {
                        // [(x || y) && (!x)] -> y
                        if *x == *operand {
                            return y.clone();
                        }
                        // [(x || y) && (!y)] -> x
                        if *y == *operand {
                            return x.clone();
                        }
                    }
                }
                _ => (),
            }
            match &other.expression {
                Expression::And { left: x, right: y } => {
                    // [x && (x && y)] -> x && y
                    // [y && (x && y)] -> x && y
                    if *x == *self || *y == *self {
                        return other.clone();
                    }
                }
                Expression::LogicalNot { operand } if *operand == *self => {
                    // [x && !x] -> false
                    return Rc::new(SymbolicValue::new_false());
                }
                Expression::Or { left: x, right: y } => {
                    // [x && (x || y)] -> x
                    // [y && (x || y)] -> y
                    if *x == *self || *y == *self {
                        return self.clone();
                    }
                    if let Expression::LogicalNot { operand } = &self.expression {
                        // [(!x) && (x || y)] -> y
                        if *x == *operand {
                            return y.clone();
                        }
                        // [(!y) && (x || y) ] -> x
                        if *y == *operand {
                            return x.clone();
                        }
                    }
                    // [x && (x && y || x && z)] -> x && (y || z)
                    if let (
                        Expression::And { left: x1, right: y },
                        Expression::And { left: x2, right: z },
                    ) = (&x.expression, &y.expression)
                    {
                        if *self == *x1 && *self == *x2 {
                            return self.and(y.or(z.clone()));
                        }
                    }
                }
                _ => (),
            }
            match (&self.expression, &other.expression) {
                // [!x && !y] -> !(x || y)
                (Expression::LogicalNot { operand: x }, Expression::LogicalNot { operand: y }) => {
                    return x.or(y.clone()).logical_not();
                }
                // [!(x && y) && x] -> x
                // [!(x && y) && y] -> y
                (Expression::LogicalNot { operand }, _) => {
                    if let Expression::And { left: x, right: y } = &operand.expression {
                        if *x == other || *y == other {
                            return other;
                        }
                    }
                }

                // [(x || (y && z)) && y] -> [(x && y) || (y && z && y)] -> (x && y) || (y && z)
                (Expression::Or { left: x, right: yz }, y) => {
                    if let Expression::And { left: y1, right: z } = &yz.expression {
                        if y1.expression == *y {
                            return x.and(y1.clone()).or(y1.and(z.clone()));
                        }
                    }
                }
                _ => (),
            }

            let other = if self_bool.is_none() {
                other.refine_with(self, 7)
            } else {
                other
            };
            SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::And {
                left,
                right,
            })
        }
    }

    /// The Boolean value of this expression, if known, otherwise None.
    fn as_bool_if_known(&self) -> Option<bool> {
        match &self.expression {
            Expression::CompileTimeConstant(ConstantValue::Int(val)) if *val == 1 => Some(true),
            Expression::CompileTimeConstant(ConstantValue::Int(val)) if *val == 0 => Some(false),
            _ => {
                // todo: ask other domains about this (construct some if need be).
                None
            }
        }
    }

    /// If the concrete Boolean value of this abstract value is known, return it as a integer constant,
    /// otherwise return None.
    fn as_int_if_known(&self) -> Option<Integer> {
        match &self.expression {
            Expression::CompileTimeConstant(ConstantValue::Int(val)) => Some(val.clone()),
            _ => None,
        }
    }

    fn cast(&self, target_type: ExpressionType) -> Rc<SymbolicValue> {
        match &self.expression {
            Expression::CompileTimeConstant(v1) => {
                let result = v1.cast(&target_type);
                if result != ConstantValue::Bottom {
                    return Rc::new(result.into());
                } else {
                    self.clone()
                }
            }
            Expression::Bottom => self.clone(),
            Expression::Join { left, right } => {
                left.cast(target_type.clone()).join(right.cast(target_type))
            }
            _ => {
                match &self.expression {
                    // [(x as t1) as target_type] -> x as target_type if t1.max_value() >= target_type.max_value()
                    Expression::Cast {
                        operand,
                        target_type: t1,
                    } => {
                        if t1.is_integer()
                            && target_type.is_unsigned_integer()
                            && t1
                                .max_value()
                                .greater_or_equal(&target_type.max_value())
                                .as_bool_if_known()
                                .unwrap_or(false)
                        {
                            return operand.cast(target_type);
                        }
                    }
                    _ => (),
                }
                if self.expression.infer_type() != target_type {
                    SymbolicValue::make_typed_unary(
                        self.clone(),
                        target_type,
                        |operand, target_type| Expression::Cast {
                            operand,
                            target_type,
                        },
                    )
                } else {
                    self.clone()
                }
            }
        }
    }

    /// Returns an element that is "*self".
    fn dereference(&self, target_type: ExpressionType) -> Rc<SymbolicValue> {
        match &self.expression {
            Expression::Bottom | Expression::Top => self.clone(),
            Expression::Cast {
                operand,
                target_type: _,
            } => operand.dereference(target_type),
            Expression::CompileTimeConstant(..) => self.clone(),
            Expression::Reference(path) => {
                if let PathEnum::HeapBlock { value } = &path.value {
                    value.clone()
                } else {
                    SymbolicValue::make_from(
                        Expression::Variable {
                            path: path.clone(),
                            var_type: target_type,
                        },
                        1,
                    )
                }
            }
            Expression::Variable { path, .. } => SymbolicValue::make_from(
                Expression::Variable {
                    path: Path::new_qualified(path.clone(), Rc::new(PathSelector::Deref)),
                    var_type: target_type,
                },
                1,
            ),
            Expression::Widen { path, operand } => operand.dereference(target_type).widen(path),
            _ => {
                info!(
                    "found unhandled expression that is of type reference: {:?}",
                    self.expression
                );
                SymbolicValue::make_typed_unknown(target_type)
            }
        }
    }

    /// Returns an element that is "self == other".
    fn equals(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        match (&self.expression, &other.expression) {
            (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) => {
                return Rc::new(v1.equals(v2).into());
            }
            // If self and other are the same location in memory, return true unless the value might be NaN.
            (
                Expression::Variable {
                    path: p1,
                    var_type: _t1,
                },
                Expression::Variable {
                    path: p2,
                    var_type: _t2,
                },
            ) => {
                if p1 == p2 {
                    return Rc::new(SymbolicValue::new_true());
                }
            }
            // [!x == 0] -> x when x is Boolean. Canonicalize it to the latter.
            (
                Expression::LogicalNot { operand },
                Expression::CompileTimeConstant(ConstantValue::Int(val)),
            ) => {
                if *val == 0 && operand.expression.infer_type() == ExpressionType::Bool {
                    return operand.clone();
                }
            }
            // [x == 0] -> !x when x is a Boolean. Canonicalize it to the latter.
            // [x == 1] -> x when x is a Boolean. Canonicalize it to the latter.
            (x, Expression::CompileTimeConstant(ConstantValue::Int(val))) => {
                if x.infer_type() == ExpressionType::Bool {
                    if *val == 0 {
                        return self.logical_not();
                    } else if *val == 1 {
                        return self.clone();
                    }
                }
            }
            (x, y) => {
                // If self and other are the same expression and the expression could not result in NaN
                // and the expression represents exactly one value, we can simplify this to true.
                if x == y {
                    return Rc::new(SymbolicValue::new_true());
                }
            }
        }
        // Return an equals expression rather than a constant expression.
        SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::Equals {
            left,
            right,
        })
    }

    /// Returns an element that is "self >= other".
    fn greater_or_equal(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        if let (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) =
            (&self.expression, &other.expression)
        {
            return Rc::new(v1.greater_or_equal(v2).into());
        };
        SymbolicValue::make_binary(self.clone(), other, |left, right| {
            Expression::GreaterOrEqual { left, right }
        })
    }

    /// Returns an element that is "self > other".
    fn greater_than(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        if let (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) =
            (&self.expression, &other.expression)
        {
            return Rc::new(v1.greater_than(v2).into());
        };
        SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::GreaterThan {
            left,
            right,
        })
    }

    /// Returns true if "self => other" is known at compile time to be true.
    /// Returning false does not imply the implication is false, just that we do not know.
    ///
    /// Important: keep the performance of this function proportional to the size of self.
    fn implies(&self, other: &Rc<SymbolicValue>) -> bool {
        // x => true, is always true
        // false => x, is always true
        // x => x, is always true
        if other.as_bool_if_known().unwrap_or(false)
            || !self.as_bool_if_known().unwrap_or(true)
            || self.eq(other)
        {
            return true;
        }

        // x && y => x
        // y && x => x
        if let Expression::And { left, right } = &self.expression {
            return left.implies(other) || right.implies(other);
        }
        false
    }

    /// Returns true if "self => !other" is known at compile time to be true.
    /// Returning false does not imply the implication is false, just that we do not know.
    fn implies_not(&self, other: &Rc<SymbolicValue>) -> bool {
        // x => !false, is always true
        // false => !x, is always true
        if !other.as_bool_if_known().unwrap_or(true) || !self.as_bool_if_known().unwrap_or(true) {
            return true;
        };
        // !x => !x
        if let Expression::LogicalNot { ref operand } = self.expression {
            return (**operand).eq(other);
        }
        false
    }

    /// Returns true if "!self => other" is known at compile time to be true.
    /// Returning false does not imply the implication is false, just that we do not know.
    fn inverse_implies(&self, other: &Rc<SymbolicValue>) -> bool {
        if let Expression::LogicalNot { operand } = &self.expression {
            return operand.implies(other);
        }
        if let Expression::LogicalNot { operand } = &other.expression {
            return self.inverse_implies_not(operand);
        }
        // x => true, is always true
        // false => x, is always true
        if other.as_bool_if_known().unwrap_or(false) || self.as_bool_if_known().unwrap_or(false) {
            return true;
        }
        false
    }

    /// Returns true if "!self => !other" is known at compile time to be true.
    /// Returning false does not imply the implication is false, just that we do not know.
    fn inverse_implies_not(&self, other: &Rc<SymbolicValue>) -> bool {
        if self == other {
            return true;
        }
        if let Expression::And { left, right } = &other.expression {
            return self.inverse_implies_not(left) || self.implies_not(right);
        }
        false
    }

    /// True if the set of concrete values that correspond to this domain is empty.
    fn is_bottom(&self) -> bool {
        matches!(&self.expression, Expression::Bottom)
    }

    /// True if this value is a compile time constant.
    fn is_compile_time_constant(&self) -> bool {
        matches!(&self.expression, Expression::CompileTimeConstant(..))
    }

    // /// True if the storage referenced by this expression is, or is contained in, a zeroed heap allocation.
    // fn is_contained_in_zeroed_heap_block(&self) -> bool {
    //     match &self.expression {
    //         Expression::HeapBlock { is_zeroed, .. } => *is_zeroed,
    //         Expression::Reference(path) | Expression::Variable { path, .. } => {
    //             path.is_rooted_by_zeroed_heap_block()
    //         }
    //         _ => false,
    //     }
    // }

    /// True if the value is derived from one or more memory locations whose values were not known
    /// when the value was constructed.
    fn is_path_alias(&self) -> bool {
        matches!(
            &self.expression,
            Expression::Reference(..) | Expression::Variable { .. } | Expression::Widen { .. }
        )
    }

    /// True if all possible concrete values are elements of the set corresponding to this domain.
    fn is_top(&self) -> bool {
        matches!(self.expression, Expression::Top)
    }

    fn is_widened(&self) -> bool {
        matches!(self.expression, Expression::Widen { .. })
    }

    /// Returns a domain whose corresponding set of concrete values include all of the values
    /// corresponding to self and other. In effect this behaves like set union.
    fn join(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        // [{} union y] -> y
        if self.is_bottom() {
            return other;
        }
        // [TOP union y] -> TOP
        if self.is_top() {
            return self.clone();
        }
        // [x union {}] -> x
        if other.is_bottom() {
            return self.clone();
        }
        // [x union x] -> x
        if (*self) == other {
            return other;
        }
        // [x union TOP] -> TOP
        if other.is_top() {
            return other;
        }
        // [widened(x) union y] -> widened(x)
        if let Expression::Widen { .. } = &self.expression {
            return self.clone();
        }
        // [x union widened(y)] -> widened(y)
        if let Expression::Widen { .. } = &other.expression {
            return other.clone();
        }
        let expression_size = self.expression_size.saturating_add(other.expression_size);
        SymbolicValue::make_from(
            Expression::Join {
                left: self.clone(),
                right: other,
            },
            expression_size,
        )
    }

    /// Returns an element that is "self <= other".
    fn less_or_equal(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        if let (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) =
            (&self.expression, &other.expression)
        {
            return Rc::new(v1.less_or_equal(v2).into());
        };
        SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::LessOrEqual {
            left,
            right,
        })
    }

    /// Returns an element that is self < other
    fn less_than(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        if let (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) =
            (&self.expression, &other.expression)
        {
            return Rc::new(v1.less_than(v2).into());
        };
        SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::LessThan {
            left,
            right,
        })
    }

    /// Returns an element that is "!self" where self is a bool.
    fn logical_not(&self) -> Rc<SymbolicValue> {
        if let Expression::CompileTimeConstant(v1) = &self.expression {
            let result = v1.logical_not();
            if result != ConstantValue::Bottom {
                return Rc::new(result.into());
            }
        };
        match &self.expression {
            Expression::Bottom => self.clone(),
            Expression::Equals { left: x, right: y } if x.expression.infer_type().is_integer() => {
                // [!(x == y)] -> x != y
                x.not_equals(y.clone())
            }
            Expression::GreaterThan { left: x, right: y }
                if x.expression.infer_type().is_integer() =>
            {
                // [!(x > y)] -> x <= y
                x.less_or_equal(y.clone())
            }
            Expression::GreaterOrEqual { left: x, right: y }
                if x.expression.infer_type().is_integer() =>
            {
                // [!(x >= y)] -> x < y
                x.less_than(y.clone())
            }
            Expression::LessThan { left: x, right: y }
                if x.expression.infer_type().is_integer() =>
            {
                // [!(x < y)] -> x >= y
                x.greater_or_equal(y.clone())
            }
            Expression::LessOrEqual { left: x, right: y }
                if x.expression.infer_type().is_integer() =>
            {
                // [!(x <= y)] -> x > y
                x.greater_than(y.clone())
            }
            Expression::LogicalNot { operand } => {
                // [!!x] -> x
                operand.clone()
            }
            Expression::Ne { left: x, right: y } if x.expression.infer_type().is_integer() => {
                // [!(x != y)] -> x == y
                x.equals(y.clone())
            }
            _ => SymbolicValue::make_unary(self.clone(), |operand| Expression::LogicalNot {
                operand,
            }),
        }
    }

    /// Returns an element that is "self != other".
    fn not_equals(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        if let (Expression::CompileTimeConstant(v1), Expression::CompileTimeConstant(v2)) =
            (&self.expression, &other.expression)
        {
            return Rc::new(v1.not_equals(v2).into());
        };
        SymbolicValue::make_binary(self.clone(), other, |left, right| Expression::Ne {
            left,
            right,
        })
    }

    /// Returns an element that is "self || other".
    fn or(&self, other: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
        fn unsimplified(x: &Rc<SymbolicValue>, y: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
            SymbolicValue::make_binary(x.clone(), y, |left, right| Expression::Or { left, right })
        }
        fn is_contained_in(x: &Rc<SymbolicValue>, y: &Rc<SymbolicValue>) -> bool {
            if *x == *y {
                return true;
            }
            if let Expression::Or { left, right } = &y.expression {
                is_contained_in(x, left) || is_contained_in(x, right)
            } else {
                false
            }
        }

        let self_as_bool = self.as_bool_if_known();
        if !self_as_bool.unwrap_or(true) {
            // [false || y] -> y
            other
        } else if self_as_bool.unwrap_or(false) || other.as_bool_if_known().unwrap_or(false) {
            // [x || true] -> true
            // [true || y] -> true
            Rc::new(SymbolicValue::new_true())
        } else if other.is_top() || other.is_bottom() || !self.as_bool_if_known().unwrap_or(true) {
            // [self || TOP] -> TOP
            // [self || BOTTOM] -> BOTTOM
            // [false || other] -> other
            other
        } else if self.is_top() || self.is_bottom() || !other.as_bool_if_known().unwrap_or(true) {
            // [TOP || other] -> TOP
            // [BOTTOM || other] -> BOTTOM
            // [self || false] -> self
            self.clone()
        } else {
            // [x || x] -> x
            if self.expression == other.expression {
                return other;
            }

            // [!x || x] -> true
            if let Expression::LogicalNot { operand } = &self.expression {
                if is_contained_in(operand, &other) {
                    return Rc::new(SymbolicValue::new_true());
                }
            }

            // [x || !x] -> true
            if let Expression::LogicalNot { operand } = &other.expression {
                if is_contained_in(operand, &self) {
                    return Rc::new(SymbolicValue::new_true());
                }
            }

            // [x || (x || y)] -> x || y
            // [x || (y || x)] -> x || y
            // [(x || y) || y] -> x || y
            // [(x || y) || x] -> x || y
            if is_contained_in(self, &other) {
                return other;
            } else if is_contained_in(&other, self) {
                return self.clone();
            }

            // [self || (x && y)] -> self || y if !self => x
            if let Expression::And { left, right: y } = &other.expression {
                if self.inverse_implies(left) {
                    return self.or(y.clone());
                }
            }

            // [x || (x && y)] -> x, etc.
            if self.inverse_implies_not(&other) {
                return self.clone();
            }

            match (&self.expression, &other.expression) {
                // [!x || x] -> true
                (Expression::LogicalNot { ref operand }, _) if (**operand).eq(&other) => {
                    Rc::new(SymbolicValue::new_true())
                }
                // [x || !x] -> true
                (_, Expression::LogicalNot { ref operand }) if (**operand).eq(&self) => {
                    Rc::new(SymbolicValue::new_true())
                }

                // [(x && y) || (x && !y)] -> x
                // [(x && y1) || (x && y2)] -> (x && (y1 || y2))
                // [(x && y1) || ((x && x3) && y2)] -> x && (y1 || (x3 && y2))
                (
                    Expression::And {
                        left: x1,
                        right: y1,
                    },
                    Expression::And {
                        left: x2,
                        right: y2,
                    },
                ) => {
                    if x1 == x2 {
                        if y1.logical_not().eq(y2) {
                            x1.clone()
                        } else {
                            x1.and(y1.or(y2.clone()))
                        }
                    } else if y1 == y2 {
                        // [(x1 && y) || (x2 && y)] -> (x1 || x2) && y
                        x1.or(x2.clone()).and(y1.clone())
                    } else {
                        if let Expression::And {
                            left: x2,
                            right: x3,
                        } = &x2.expression
                        {
                            if x1 == x2 {
                                return x1.and(y1.or(x3.and(y2.clone())));
                            }
                        }
                        unsimplified(self, other)
                    }
                }

                // [((c ? e : 1) == 1) || ((c ? e : 1) == 0)] -> !c || e == 0 || e == 1
                (
                    Expression::Equals {
                        left: l1,
                        right: r1,
                    },
                    Expression::Equals {
                        left: l2,
                        right: r2,
                    },
                ) if l1 == l2 && r1.expression.is_one() && r2.expression.is_zero() => {
                    unsimplified(self, other)
                }

                // [(x && y) || x] -> x
                // [(x && y) || y] -> y
                (Expression::And { left: x, right: y }, _) if *x == other || *y == other => other,

                // [x || (x && y)] -> x
                // [y || (x && y)] -> y
                (_, Expression::And { left: x, right: y }) if *x == *self || *y == *self => {
                    self.clone()
                }

                // [x || (!x && z)] -> x || z
                (_, Expression::And { left: y, right: z }) if self.inverse_implies(y) => {
                    self.or(z.clone())
                }

                // [(x && y) || (!x || !y)] -> true
                (Expression::And { left: x, right: y }, Expression::Or { left, right })
                    if x.inverse_implies(left) && y.inverse_implies(right) =>
                {
                    Rc::new(SymbolicValue::new_true())
                }

                // [(x && !y) || y] -> (y || x)
                (Expression::And { left: x, right }, _) => match &right.expression {
                    Expression::LogicalNot { operand: y } if *y == other => y.or(x.clone()),
                    _ => unsimplified(self, other),
                },

                // [x || !(x || y)] -> x || !y
                (_, Expression::LogicalNot { operand }) => match &operand.expression {
                    Expression::Or { left: x2, right: y } if *self == *x2 => {
                        self.or(y.logical_not())
                    }
                    _ => unsimplified(self, other),
                },

                _ => unsimplified(self, other),
            }
        }
    }

    /// Adds any abstract heap addresses found in the associated expression to the given set.
    fn record_heap_blocks(&self, result: &mut HashSet<Rc<SymbolicValue>>) {
        self.expression.record_heap_blocks(result);
    }

    /// True if all of the concrete values that correspond to self also correspond to other.
    /// Note: !x.subset(y) does not imply y.subset(x).
    fn subset(&self, other: &Rc<SymbolicValue>) -> bool {
        if self == other {
            return true;
        };
        match (&self.expression, &other.expression) {
            // The empty set is a subset of every other set.
            (Expression::Bottom, _) => true,
            // A non empty set is not a subset of the empty set.
            (_, Expression::Bottom) => false,
            // Every set is a subset of the universal set.
            (_, Expression::Top) => true,
            // The universal set is not a subset of any set other than the universal set.
            (Expression::Top, _) => false,
            // Widened expressions are equal if their paths are equal, regardless of their operand values.
            (Expression::Widen { path: p1, .. }, Expression::Widen { path: p2, .. }) => *p1 == *p2,
            // x subset widen { z } if x subset z
            (_, Expression::Widen { operand, .. }) => self.subset(&operand),
            // (left join right) is a subset of x if both left and right are subsets of x.
            (Expression::Join { left, right, .. }, _) => {
                // This is a conservative answer. False does not imply other.subset(self).
                left.subset(other) && right.subset(other)
            }
            // x is a subset of (left join right) if x is a subset of either left or right.
            (_, Expression::Join { left, right, .. }) => {
                // This is a conservative answer. False does not imply other.subset(self).
                self.subset(&left) || self.subset(&right)
            }
            // in all other cases we conservatively answer false
            _ => false,
        }
    }

    /// Returns a domain that is simplified (refined) by using the current path conditions
    /// (conditions known to be true in the current context). If no refinement is possible
    /// the result is simply a clone of this domain.
    ///
    /// This function is performance critical and involves a tricky trade-off: Invoking it
    /// is expensive, particularly when expressions get large (hence k_limits::MAX_EXPRESSION_SIZE).
    /// One reason for this is that expressions are traversed without doing any kind of occurs check,
    /// so expressions that are not large in memory usage (because of sharing) can still be too large
    /// to traverse. Currently there is no really efficient way to add an occurs check, so the
    /// k-limit approach is cheaper, at the cost of losing precision.
    ///
    /// On the other hand, getting rid of this refinement (and the k-limits it needs) will cause
    /// a lot of expressions to get much larger because of joining and composing. This will increase
    /// the cost of refine_parameters, which is essential. Likewise, it wil also increase the cost
    /// of refine_paths, which ensures that paths stay unique (dealing with aliasing is expensive).
    fn refine_with(&self, path_condition: &Self, depth: usize) -> Rc<SymbolicValue> {
        //do not use false path conditions to refine things
        if depth >= k_limits::MAX_REFINE_DEPTH {
            //todo: perhaps this should go away.
            // right now it deals with the situation where some large expressions have sizes
            // that are not accurately tracked. These really should get fixed.
            return self.clone();
        }
        // In this context path_condition is true
        if path_condition.eq(self) {
            return Rc::new(SymbolicValue::new_true());
        }

        // If the path context constrains the self expression to be equal to a constant, just
        // return the constant.
        if let Expression::Equals { left, right } = &path_condition.expression {
            if let Expression::CompileTimeConstant(..) = &left.expression {
                if self.eq(right) {
                    return left.clone();
                }
            }
            if let Expression::CompileTimeConstant(..) = &right.expression {
                if self.eq(left) {
                    return right.clone();
                }
            }
        }
        // Traverse the self expression, looking for recursive refinement opportunities.
        // Important, keep the traversal as trivial as possible and put optimizations in
        // the transfer functions. Also, keep the transfer functions constant in cost as
        // much as possible. Any time they are not, this function becomes quadratic and
        // performance becomes terrible.
        match &self.expression {
            Expression::Drop(..) => self.clone(),
            Expression::Numerical(..) => self.clone(),
            Expression::Bottom | Expression::Top => self.clone(),
            Expression::And { left, right } => left
                .refine_with(path_condition, depth + 1)
                .and(right.refine_with(path_condition, depth + 1)),
            Expression::Cast {
                operand,
                target_type,
            } => operand
                .refine_with(path_condition, depth + 1)
                .cast(target_type.clone()),
            Expression::CompileTimeConstant(..) => self.clone(),
            Expression::Equals { left, right } => left
                .refine_with(path_condition, depth + 1)
                .equals(right.refine_with(path_condition, depth + 1)),
            Expression::GreaterOrEqual { left, right } => left
                .refine_with(path_condition, depth + 1)
                .greater_or_equal(right.refine_with(path_condition, depth + 1)),
            Expression::GreaterThan { left, right } => left
                .refine_with(path_condition, depth + 1)
                .greater_than(right.refine_with(path_condition, depth + 1)),
            Expression::HeapBlock { .. } => self.clone(),
            Expression::Join { left, right } => left
                .refine_with(path_condition, depth + 1)
                .join(right.refine_with(path_condition, depth + 1)),
            Expression::LessOrEqual { left, right } => left
                .refine_with(path_condition, depth + 1)
                .less_or_equal(right.refine_with(path_condition, depth + 1)),
            Expression::LessThan { left, right } => left
                .refine_with(path_condition, depth + 1)
                .less_than(right.refine_with(path_condition, depth + 1)),
            Expression::Ne { left, right } => left
                .refine_with(path_condition, depth + 1)
                .not_equals(right.refine_with(path_condition, depth + 1)),
            Expression::LogicalNot { operand } => {
                operand.refine_with(path_condition, depth + 1).logical_not()
            }
            Expression::Or { left, right } => {
                // Ideally the constructor should do the simplifications, but in practice or
                // expressions grow quite large due to composition and it really helps to avoid
                // refining the right expression whenever possible, even at the expense of
                // more checks here. If the performance of implies and implies_not should become
                // significantly worse than it is now, this could become a performance bottle neck.
                if path_condition.implies(&left) || path_condition.implies(&right) {
                    Rc::new(SymbolicValue::new_true())
                } else if path_condition.implies_not(&left) {
                    if path_condition.implies_not(&right) {
                        Rc::new(SymbolicValue::new_false())
                    } else {
                        right.refine_with(path_condition, depth + 1)
                    }
                } else if path_condition.implies_not(&right) {
                    left.refine_with(path_condition, depth + 1)
                } else {
                    left.refine_with(path_condition, depth + 1)
                        .or(right.refine_with(path_condition, depth + 1))
                }
            }
            Expression::Reference(..) => self.clone(),
            Expression::Variable { var_type, .. } => {
                if *var_type == ExpressionType::Bool {
                    if path_condition.implies(&self) {
                        return Rc::new(SymbolicValue::new_true());
                    } else if path_condition.implies_not(&self) {
                        return Rc::new(SymbolicValue::new_false());
                    }
                }
                self.clone()
            }
            Expression::Widen { path, operand } => {
                operand.refine_with(path_condition, depth + 1).widen(&path)
            }
        }
    }

    /// Returns a domain whose corresponding set of concrete values include all of the values
    /// corresponding to self and other. The set of values may be less precise (more inclusive) than
    /// the set returned by join. The chief requirement is that a small number of widen calls
    /// deterministically lead to a set of values that include of the values that could be stored
    /// in memory at the given path.
    fn widen(&self, path: &Rc<Path>) -> Rc<SymbolicValue> {
        match &self.expression {
            Expression::CompileTimeConstant(..)
            | Expression::HeapBlock { .. }
            | Expression::Reference(..)
            | Expression::Top
            | Expression::Variable { .. }
            | Expression::Widen { .. } => self.clone(),
            _ => {
                if self.expression_size > 1000 {
                    SymbolicValue::make_from(
                        Expression::Variable {
                            path: path.clone(),
                            var_type: self.expression.infer_type(),
                        },
                        1,
                    )
                } else {
                    SymbolicValue::make_from(
                        Expression::Widen {
                            path: path.clone(),
                            operand: self.clone(),
                        },
                        3,
                    )
                }
            }
        }
    }
}

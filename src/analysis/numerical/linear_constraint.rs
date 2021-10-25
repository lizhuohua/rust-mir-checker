use crate::analysis::memory::expression::Expression;
use crate::analysis::memory::path::{Path, PathEnum};
use crate::analysis::memory::symbolic_value::SymbolicValue;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::lattice::LatticeTrait;
use apron_sys;
use foreign_types::ForeignType;
use rug::Integer;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{self, Debug};
use std::ops::{Add, Mul, Neg, Sub};
use std::rc::Rc;

/// Represents a linear expression with integer coefficients
/// E.g. 3*a+4*b-5*c+6, where `cof_map` stores {(a,3), (b,4), (c,-5)}, and `cst` stores 6
#[derive(PartialEq, Eq, Clone)]
pub struct LinearExpression {
    cof_map: BTreeMap<Rc<Path>, Integer>,
    cst: Integer,
}

impl Default for LinearExpression {
    /// The default value of a linear expression is simply zero
    fn default() -> Self {
        Self {
            cof_map: BTreeMap::new(),
            cst: Integer::from(0),
        }
    }
}

impl LinearExpression {
    /// Test if the expression only has the constant term
    pub fn is_constant(&self) -> bool {
        self.cof_map.is_empty()
    }

    /// Returns the constant term
    pub fn constant(&self) -> Integer {
        self.cst.clone()
    }

    /// Get the coefficient of variable `var`, e.g. for linear expression `3a+4b-5c`,
    /// the coefficient of variable `a` is 3. If `var` is not found in the expression,
    /// returns zero
    pub fn get_coff(&self, var: Rc<Path>) -> Integer {
        if let Some(coff) = self.cof_map.get(&var) {
            coff.clone()
        } else {
            Integer::from(0)
        }
    }

    /// Add term `n*x` to the linear expression
    pub fn add_term(&mut self, x: Rc<Path>, n: Integer) {
        if let Some(num) = self.cof_map.get(&x) {
            let r = num + n;
            if r == 0 {
                self.cof_map.remove(&x);
            } else {
                self.cof_map.insert(x.clone(), r);
            }
        } else if n != 0 {
            self.cof_map.insert(x.clone(), n);
        }
    }
}

impl<Num> From<Num> for LinearExpression
where
    Integer: From<Num>,
{
    fn from(src: Num) -> Self {
        LinearExpression::default() + Integer::from(src)
    }
}

impl Add<Self> for LinearExpression {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut res = Self {
            cof_map: self.cof_map,
            cst: self.cst + &other.cst,
        };
        for (var, coff) in other {
            res.add_term(var, coff);
        }
        res
    }
}

impl Add<Integer> for LinearExpression {
    type Output = Self;

    fn add(self, other: Integer) -> Self {
        Self {
            cof_map: self.cof_map,
            cst: self.cst + other,
        }
    }
}

impl Add<Rc<Path>> for LinearExpression {
    type Output = Self;

    fn add(self, other: Rc<Path>) -> Self {
        let mut res = Self {
            cof_map: self.cof_map,
            cst: self.cst,
        };
        res.add_term(other, Integer::from(1));
        res
    }
}

impl Sub<Self> for LinearExpression {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut res = Self {
            cof_map: self.cof_map,
            cst: self.cst - &other.cst,
        };
        for (var, coff) in other {
            res.add_term(var, -coff);
        }
        res
    }
}

impl Sub<Integer> for LinearExpression {
    type Output = Self;

    fn sub(self, other: Integer) -> Self {
        self + (-other)
    }
}

impl Sub<Rc<Path>> for LinearExpression {
    type Output = Self;

    fn sub(self, other: Rc<Path>) -> Self {
        let mut res = Self {
            cof_map: self.cof_map,
            cst: self.cst,
        };
        res.add_term(other, Integer::from(-1));
        res
    }
}

impl Mul<Integer> for LinearExpression {
    type Output = Self;

    fn mul(self, other: Integer) -> Self {
        if other == 0 {
            Self::default()
        } else {
            let mut cof_map = BTreeMap::new();
            for (var, coff) in &self {
                let r = coff * other.clone();
                if r != 0 {
                    cof_map.insert(var.clone(), r);
                }
            }
            Self {
                cof_map,
                cst: other * &self.cst,
            }
        }
    }
}

impl Neg for LinearExpression {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self * Integer::from(-1)
    }
}

impl IntoIterator for LinearExpression {
    type Item = (Rc<Path>, Integer);
    type IntoIter = std::collections::btree_map::IntoIter<Rc<Path>, Integer>;
    fn into_iter(self) -> Self::IntoIter {
        self.cof_map.into_iter()
    }
}

impl<'a> IntoIterator for &'a LinearExpression {
    type Item = (&'a Rc<Path>, &'a Integer);
    type IntoIter = std::collections::btree_map::Iter<'a, Rc<Path>, Integer>;
    fn into_iter(self) -> Self::IntoIter {
        self.cof_map.iter()
    }
}

impl<'a> IntoIterator for &'a mut LinearExpression {
    type Item = (&'a Rc<Path>, &'a mut Integer);
    type IntoIter = std::collections::btree_map::IterMut<'a, Rc<Path>, Integer>;
    fn into_iter(self) -> Self::IntoIter {
        self.cof_map.iter_mut()
    }
}

impl Debug for LinearExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        for (i, (v, n)) in self.cof_map.iter().enumerate() {
            if *n > 0 && i != 0 {
                res.push('+');
            }
            if *n == -1 {
                res.push('-');
            } else if *n != 1 {
                res.push_str(format!("{}*", n).as_str());
            }
            res.push_str(format!("{:?}", v).as_str());
        }
        if self.cst > 0 && !self.cof_map.is_empty() {
            res.push('+');
        }
        if self.cst != 0 || self.cof_map.is_empty() {
            res.push_str(format!("{}", self.cst).as_str());
        }
        write!(f, "{}", res)
    }
}

fn refine_symbolic_value(val: Rc<SymbolicValue>) -> Rc<SymbolicValue> {
    use Expression::*;
    match &val.expression {
        Ne { left, right } => {
            if let LogicalNot {
                operand: left_operand,
            } = &left.expression
            {
                return SymbolicValue::make_from(
                    Expression::Equals {
                        left: left_operand.clone(),
                        right: right.clone(),
                    },
                    1,
                );
            }
        }
        _ => {}
    }
    return val;
}

fn symbolic_to_expression(val: Rc<SymbolicValue>) -> Result<LinearExpression, &'static str> {
    let val = refine_symbolic_value(val);
    debug!("In symbolic_to_expression() ,val:{:?}", val);
    let mut expr = LinearExpression::default();
    use Expression::*;
    match &val.expression {
        Numerical(path) => {
            expr = expr + path.clone();
        }
        CompileTimeConstant(const_value) => {
            if let Some(integer) = const_value.try_get_integer() {
                expr = expr + integer;
            }
        }
        HeapBlock { .. } => {
            let path: Path = PathEnum::HeapBlock { value: val }.into();
            expr = expr + Rc::new(path);
        }
        Variable { path, var_type } => {
            if var_type.is_integer() {
                expr = expr + path.clone();
            }
        }
        Widen { operand, .. } => {
            if let Ok(operand) = symbolic_to_expression(operand.clone()) {
                expr = expr + operand;
            } else {
                return Err(
                    "Error when try to convert a Widen symbolic value into a linear expression",
                );
            }
        }
        Cast { operand, .. } => {
            if let Ok(operand) = symbolic_to_expression(operand.clone()) {
                expr = expr + operand;
            } else {
                return Err(
                    "Error when try to convert a Cast symbolic value into a linear expression",
                );
            }
        }
        // IntrinsicBitVectorUnary { .. } => {
        //     // TODO: implement this?
        // }

        Top
        | Bottom
        | And { .. }
        | Equals { .. }
        | GreaterOrEqual { .. }
        | GreaterThan { .. }
        // | HeapBlockLayout { .. }
        | Join { .. }
        | LessOrEqual { .. }
        | LessThan { .. }
        | Or { .. }
        | Ne { .. }
        | Reference(..)
        | LogicalNot { .. }
        // | Offset { .. }
        | Drop(..) => {
            // TODO: This causes crashes, how to deal with this?
            return Err("Error when try to convert a symbolic value into a linear expression");
            // unreachable!("Error when try to convert a symbolic value into a linear expression")
        }
    }
    Ok(expr)
}

#[derive(PartialEq, Eq, Clone)]
// /// A linear constraint `cons := exp op exp | ¬ cons | cons ∧ cons | cons ∨ cons`
/// A linear constraint `cons := exp op exp`
/// Where `op` is `==`, `!=`, `<=`, `<`, other operators can be transformed into these
pub enum LinearConstraint {
    // "=="
    Equality(LinearExpression),
    // "!="
    Inequality(LinearExpression),
    // "<="
    LessEq(LinearExpression),
    // "<"
    LessThan(LinearExpression),
}

impl LinearConstraint {
    pub fn new_true() -> Self {
        Self::Equality(LinearExpression::from(0))
    }

    pub fn new_false() -> Self {
        Self::Inequality(LinearExpression::from(0))
    }

    pub fn is_tautology(&self) -> bool {
        match self {
            LinearConstraint::Equality(expr) => expr.is_constant() && expr.constant() == 0,
            LinearConstraint::Inequality(expr) => expr.is_constant() && expr.constant() != 0,
            LinearConstraint::LessEq(expr) => expr.is_constant() && expr.constant() <= 0,
            LinearConstraint::LessThan(expr) => expr.is_constant() && expr.constant() < 0,
        }
    }

    pub fn is_contradiction(&self) -> bool {
        match self {
            LinearConstraint::Equality(expr) => expr.is_constant() && expr.constant() != 0,
            LinearConstraint::Inequality(expr) => expr.is_constant() && expr.constant() == 0,
            LinearConstraint::LessEq(expr) => expr.is_constant() && expr.constant() > 0,
            LinearConstraint::LessThan(expr) => expr.is_constant() && expr.constant() >= 0,
        }
    }

    pub fn is_strict(&self) -> bool {
        matches!(self, LinearConstraint::LessThan(..))
    }

    pub fn strict_to_non_strict(&self) -> Self {
        assert!(self.is_strict());
        match self {
            Self::LessThan(expr) => Self::LessEq(expr.clone() + Integer::from(1)),
            _ => unreachable!(),
        }
    }

    pub fn negate(&self) -> Self {
        if self.is_tautology() {
            Self::new_false()
        } else if self.is_contradiction() {
            Self::new_true()
        } else {
            match self {
                LinearConstraint::Equality(expr) => Self::Inequality(expr.clone()),
                LinearConstraint::Inequality(expr) => Self::Equality(expr.clone()),
                LinearConstraint::LessEq(expr) => Self::LessThan(-expr.clone()),
                LinearConstraint::LessThan(expr) => Self::LessEq(-expr.clone()),
            }
        }
    }
}

impl From<LinearConstraint> for LinearConstraintSystem {
    fn from(value: LinearConstraint) -> Self {
        let mut result = LinearConstraintSystem::default();
        result.add(value);
        result
    }
}

/// Convert Expression::LessOrEqual/LessThan/GreaterOrEqual/GreaterThan/Equals/Ne/Variable/Numerical/Widen
/// into LinearConstraint::LessEq/LessThan/Equality/Inequality
/// Some `Expresson` cannot be converted by Apron's design, e.g. Or
/// So we do not implement them as they should never be used
/// for constructing `LinearConstraint`
impl TryFrom<Rc<SymbolicValue>> for LinearConstraintSystem {
    type Error = &'static str;
    fn try_from(value: Rc<SymbolicValue>) -> Result<Self, &'static str> {
        debug!(
            "Converting symbolic value into LinearConstraintSystem: {:?}",
            value
        );

        let res =
            match &value.expression {
                Expression::And { left, right } => {
                    let lcsts = LinearConstraintSystem::try_from(left.clone());
                    let rcsts = LinearConstraintSystem::try_from(right.clone());
                    if let (Ok(lcsts), Ok(rcsts)) = (lcsts, rcsts) {
                        lcsts.join(rcsts)
                    } else {
                        return Err("Error when converting And expression");
                    }
                }
                // Constant, so it must be either true or false
                Expression::CompileTimeConstant(const_value) => {
                    match const_value.as_bool_if_known() {
                    Some(true) => LinearConstraint::new_true().into(),
                    Some(false) => LinearConstraint::new_false().into(),
                    None => unreachable!("Converting a constant symbolic value into linear constraint but the value is neither true nor false"),
                }
                }
                // A single variable, equivalent to `var == true`
                Expression::Variable { path, .. } => {
                    let mut expr = LinearExpression::default();
                    expr = expr + path.clone() - Integer::from(1);
                    LinearConstraint::Equality(expr).into()
                }
                // A single numerical variable, equivalent to `var == true`
                Expression::Numerical(path) => {
                    let mut expr = LinearExpression::default();
                    expr = expr + path.clone() - Integer::from(1);
                    LinearConstraint::Equality(expr).into()
                }
                Expression::Widen { operand, .. } => {
                    Self::try_from(operand.clone()).unwrap().into()
                }
                // An expression `lhs <= rhs`, equivalent to `lhs - rhs <= 0`
                Expression::LessOrEqual { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::LessEq(left_expr - right_expr).into()
                    } else {
                        return Err("Error when converting LessOrEqual expression");
                    }
                }
                // An expression `lhs < rhs`, equivalent to `lhs - rhs < 0`
                Expression::LessThan { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::LessThan(left_expr - right_expr).into()
                    } else {
                        return Err("Error when converting LessThan expression");
                    }
                }
                // An expression `lhs == rhs`, equivalent to `lhs - rhs == 0`
                Expression::Equals { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::Equality(left_expr - right_expr).into()
                    } else {
                        return Err("Error when converting Equals expression");
                    }
                }
                // An expression `lhs >= rhs`, equivalent to `rhs - lhs <= 0`
                Expression::GreaterOrEqual { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::LessEq(right_expr - left_expr).into()
                    } else {
                        return Err("Error when converting GreaterOrEqual expression");
                    }
                }
                // An expression `lhs > rhs`, equivalent to `rhs - lhs < 0`
                Expression::GreaterThan { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::LessThan(right_expr - left_expr).into()
                    } else {
                        return Err("Error when converting GreaterThan expression");
                    }
                }
                // An expression `lhs != rhs`, equivalent to `lhs - rhs != 0`
                Expression::Ne { left, right } => {
                    let left_expr = symbolic_to_expression(left.clone());
                    let right_expr = symbolic_to_expression(right.clone());
                    if let (Ok(left_expr), Ok(right_expr)) = (left_expr, right_expr) {
                        LinearConstraint::Inequality(left_expr - right_expr).into()
                    } else {
                        return Err("Error when converting Ne expression");
                    }
                }
                // An expression `¬ expr`, equivalent to `expr.negate()`
                Expression::LogicalNot { operand } => {
                    if let Ok(csts) = Self::try_from(operand.clone()) {
                        // There should be only 1 constraint in `operand`
                        // Because `¬(p∧q)` should never be generated. FIXME: is this correct?
                        assert!(csts.size() == 1);
                        let cst = &csts.csts[0];
                        cst.negate().into()
                    } else {
                        return Err("Error when converting LogicNot");
                    }
                }
                // `Top` means everything is possible, so it does not introduce any new constraints
                Expression::Top => LinearConstraint::new_true().into(),
                // `Bottom` is equivalent to false, FIXME: is this correct?
                Expression::Bottom => LinearConstraint::new_false().into(),
                Expression::Reference(..) => return Err("reference"),
                // Expression::Offset { .. } => return Err("offset"),
                Expression::Or { .. } => return Err("or"),
                Expression::Join { .. } => return Err("join"),
                // Expression::IntrinsicBitVectorUnary { .. } => return Err("intrinsic"),
                Expression::HeapBlock { .. } => return Err("heap"),
                _ => return Err("The SymbolicValue cannot be converted into LinearConstraint"),
            };
        debug!("Convert result: {:?}", res);
        Ok(res)
    }
}

impl Debug for LinearConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_contradiction() {
            write!(f, "false")
        } else if self.is_tautology() {
            write!(f, "true")
        } else {
            let e;
            let op;
            let c;
            match self {
                LinearConstraint::Equality(expr) => {
                    e = expr.clone() - expr.constant();
                    op = "=";
                    c = -expr.constant();
                }
                LinearConstraint::Inequality(expr) => {
                    e = expr.clone() - expr.constant();
                    op = "!=";
                    c = -expr.constant();
                }
                LinearConstraint::LessEq(expr) => {
                    e = expr.clone() - expr.constant();
                    op = "<=";
                    c = -expr.constant();
                }
                LinearConstraint::LessThan(expr) => {
                    e = expr.clone() - expr.constant();
                    op = "<";
                    c = -expr.constant();
                }
            }
            write!(f, "{:?}{}{}", e, op, c)
        }
    }
}

#[derive(Clone)]
pub struct LinearConstraintSystem {
    csts: Vec<LinearConstraint>,
}

impl Default for LinearConstraintSystem {
    fn default() -> Self {
        Self { csts: Vec::new() }
    }
}

impl LinearConstraintSystem {
    pub fn add(&mut self, cst: LinearConstraint) {
        if !self.csts.iter().any(|constraint| *constraint == cst) {
            self.csts.push(cst);
        }
    }

    pub fn join(&self, csts: LinearConstraintSystem) -> Self {
        let mut result = Self::default();
        for cst in csts {
            result.add(cst);
        }
        result
    }

    pub fn size(&self) -> usize {
        self.csts.len()
    }

    pub fn is_false(&self) -> bool {
        if self.csts.is_empty() {
            false
        } else {
            for cst in &self.csts {
                if !cst.is_contradiction() {
                    return false;
                }
            }
            true
        }
    }

    pub fn is_true(&self) -> bool {
        self.csts.is_empty()
    }
}

impl IntoIterator for LinearConstraintSystem {
    type Item = LinearConstraint;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.csts.into_iter()
    }
}

impl<'a> IntoIterator for &'a LinearConstraintSystem {
    type Item = &'a LinearConstraint;
    type IntoIter = std::slice::Iter<'a, LinearConstraint>;
    fn into_iter(self) -> Self::IntoIter {
        self.csts.iter()
    }
}

impl<'a> IntoIterator for &'a mut LinearConstraintSystem {
    type Item = &'a mut LinearConstraint;
    type IntoIter = std::slice::IterMut<'a, LinearConstraint>;
    fn into_iter(self) -> Self::IntoIter {
        self.csts.iter_mut()
    }
}

impl Debug for LinearConstraintSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.csts.is_empty() {
            write!(f, "{{}}")
        } else {
            let mut res = String::from("{");
            for cst in self {
                res.push_str(format!("{:?}; ", cst).as_str());
            }
            res.pop();
            res.pop();
            res.push('}');
            write!(f, "{}", res)
        }
    }
}

impl<Type> From<&ApronAbstractDomain<Type>> for LinearConstraintSystem
where
    Type: ApronDomainType,
    ApronAbstractDomain<Type>: GetManagerTrait,
{
    fn from(inv: &ApronAbstractDomain<Type>) -> Self {
        let mut cst_system = Self::default();
        if inv.is_bottom() {
            cst_system.add(LinearConstraint::new_false());
        } else if inv.is_top() {
            cst_system.add(LinearConstraint::new_true());
        } else {
            let mut cons_array = unsafe {
                apron_sys::ap_abstract0_to_lincons_array(
                    ApronAbstractDomain::<Type>::get_manager().as_ptr(),
                    inv.get_state().as_ptr(),
                )
            };
            for i in 0..cons_array.size {
                unsafe {
                    cst_system.add(inv.apcons2cons(*cons_array.p.add(i)));
                }
            }
            unsafe {
                apron_sys::ap_lincons0_array_clear(
                    &mut cons_array as *mut apron_sys::ap_lincons0_array_t,
                );
            }
        }
        cst_system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_expression() {
        let mut exp1 = LinearExpression::default();
        let mut exp2 = LinearExpression::default();
        let x = Path::new_local(1, 0);
        let y = Path::new_local(2, 0);
        // exp1 = 3x
        exp1.add_term(x.clone(), Integer::from(3));
        // exp2 = 4y
        exp2.add_term(y.clone(), Integer::from(4));
        // exp1 = exp1 * 5 = 15x
        exp1 = exp1 * Integer::from(5);
        // exp1 = exp1 + exp2 * 1 = 15x + 4y
        exp1 = exp1 + exp2.clone() * Integer::from(1);
        println!("exp1 = {:?}", exp1);
        let exp4 = exp1.clone() - exp2;
        println!("exp4 = {:?}", exp4);

        let mut exp3 = LinearExpression::default();
        exp3.add_term(y, Integer::from(4));
        exp3.add_term(x, Integer::from(15));
        println!("exp3 = {:?}", exp3);

        assert_eq!(exp1, exp3);
    }
}

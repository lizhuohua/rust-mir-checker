use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::{Expression, ExpressionType};
use crate::analysis::memory::path::Path;
use crate::analysis::memory::symbolic_value::SymbolicValue;
use crate::analysis::numerical::linear_constraint::{
    LinearConstraint, LinearConstraintSystem, LinearExpression,
};
use rug::Integer;
use std::ffi::CString;
use std::fmt;
use std::rc::Rc;
use std::sync::Mutex;
use z3_sys;

lazy_static! {
    static ref Z3_MUTEX: Mutex<()> = Mutex::new(());
}

pub type Z3Expression = z3_sys::Z3_ast;

pub struct Z3Solver {
    z3_context: z3_sys::Z3_context,
    z3_solver: z3_sys::Z3_solver,
    empty_str: z3_sys::Z3_string,
    any_sort: z3_sys::Z3_sort,
    bool_sort: z3_sys::Z3_sort,
    int_sort: z3_sys::Z3_sort,
    zero: z3_sys::Z3_ast,
    // one: z3_sys::Z3_ast,
}

impl fmt::Debug for Z3Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Z3Solver".fmt(f)
    }
}

impl Default for Z3Solver {
    fn default() -> Self {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            let z3_sys_cfg = z3_sys::Z3_mk_config();
            let time_out = CString::new("timeout").unwrap().into_raw();
            let ms = CString::new("100").unwrap().into_raw();
            z3_sys::Z3_set_param_value(z3_sys_cfg, time_out, ms);
            let z3_context = z3_sys::Z3_mk_context(z3_sys_cfg);
            let z3_solver = z3_sys::Z3_mk_solver(z3_context);
            let empty_str = CString::new("").unwrap().into_raw();
            let symbol = z3_sys::Z3_mk_string_symbol(z3_context, empty_str);

            let any_sort = z3_sys::Z3_mk_uninterpreted_sort(z3_context, symbol);
            let bool_sort = z3_sys::Z3_mk_bool_sort(z3_context);
            let int_sort = z3_sys::Z3_mk_int_sort(z3_context);
            let zero = z3_sys::Z3_mk_int(z3_context, 0, int_sort);
            // let one = z3_sys::Z3_mk_int(z3_context, 1, int_sort);
            // let two = z3_sys::Z3_mk_int(z3_context, 2, int_sort);
            Self {
                z3_context,
                z3_solver,
                empty_str,
                any_sort,
                bool_sort,
                int_sort,
                zero,
                // one,
            }
        }
    }
}

#[derive(PartialEq)]
pub enum SmtResult {
    /// There is an assignment of values to the free variables for which the expression is true.
    Sat,
    /// There is a proof that no assignment of values to the free variables can make the expression true.
    Unsat,
    /// The solver timed out while trying to solve this expression.
    Unknown,
}

impl Z3Solver {
    /// Solve constraints and return result
    pub fn solve(&self) -> SmtResult {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            match z3_sys::Z3_solver_check(self.z3_context, self.z3_solver) {
                z3_sys::Z3_L_TRUE => SmtResult::Sat,
                z3_sys::Z3_L_FALSE => SmtResult::Unsat,
                _ => SmtResult::Unknown,
            }
        }
    }

    /// Solve a Z3 ast without changing the internal state of the solver
    pub fn solve_expression(&self, expression: &Z3Expression) -> SmtResult {
        self.set_backtrack_position();
        self.assert(expression);
        let result = self.solve();
        self.backtrack();
        result
    }

    /// Add a constraint into the solver
    pub fn assert(&self, expression: &Z3Expression) {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            z3_sys::Z3_solver_assert(self.z3_context, self.z3_solver, *expression);
        }
    }

    pub fn make_or_z3_expression(
        &self,
        cst1: &LinearConstraint,
        cst2: &LinearConstraint,
    ) -> Z3Expression {
        let z3_cst1 = self.get_as_z3_expression(cst1);
        let z3_cst2 = self.get_as_z3_expression(cst2);

        let tmp = vec![z3_cst1, z3_cst2];
        unsafe { z3_sys::Z3_mk_or(self.z3_context, 2, tmp.as_ptr()) }
    }

    pub fn make_and_z3_expression(
        &self,
        cst1: &LinearConstraint,
        cst2: &LinearConstraint,
    ) -> Z3Expression {
        let z3_cst1 = self.get_as_z3_expression(cst1);
        let z3_cst2 = self.get_as_z3_expression(cst2);

        let tmp = vec![z3_cst1, z3_cst2];
        unsafe { z3_sys::Z3_mk_and(self.z3_context, 2, tmp.as_ptr()) }
    }

    pub fn make_not_z3_expression(&self, exp: Z3Expression) -> Z3Expression {
        unsafe { z3_sys::Z3_mk_not(self.z3_context, exp) }
    }

    pub fn make_ite_z3_expression(
        &self,
        t1: Z3Expression,
        t2: Z3Expression,
        t3: Z3Expression,
    ) -> Z3Expression {
        unsafe { z3_sys::Z3_mk_ite(self.z3_context, t1, t2, t3) }
    }

    /// Convert a linear constraint system into Z3 expression
    /// The AST is always of sort `bool`
    pub fn get_csts_as_z3_expression(&self, csts: &LinearConstraintSystem) -> Z3Expression {
        let mut tmp = vec![];
        let mut len: std::os::raw::c_uint = 0;
        for cst in csts {
            let z3_expr = self.get_as_z3_expression(cst);
            tmp.push(z3_expr);
            len += 1;
        }
        unsafe { z3_sys::Z3_mk_and(self.z3_context, len, tmp.as_ptr()) }
    }

    /// Convert a single linear constraint into Z3 expression
    /// The AST is always of sort `bool`
    pub fn get_as_z3_expression(&self, cst: &LinearConstraint) -> Z3Expression {
        let _guard = Z3_MUTEX.lock().unwrap();

        let zero = unsafe { z3_sys::Z3_mk_int64(self.z3_context, 0, self.int_sort) };
        match cst {
            LinearConstraint::Equality(expr) => {
                let e = self.expr2int(expr);
                unsafe { z3_sys::Z3_mk_eq(self.z3_context, e, zero) }
            }
            LinearConstraint::Inequality(expr) => {
                let e = self.expr2int(expr);
                unsafe {
                    z3_sys::Z3_mk_not(self.z3_context, z3_sys::Z3_mk_eq(self.z3_context, e, zero))
                }
            }
            LinearConstraint::LessEq(expr) => {
                let e = self.expr2int(expr);
                unsafe { z3_sys::Z3_mk_le(self.z3_context, e, zero) }
            }
            LinearConstraint::LessThan(expr) => {
                let e = self.expr2int(expr);
                unsafe { z3_sys::Z3_mk_lt(self.z3_context, e, zero) }
            }
        }
    }

    /// Convert a symbolic constraint `path==value` into Z3 AST
    /// The AST is always of sort `bool`
    pub fn get_symbolic_constraint(&self, path: &Rc<Path>, value: &SymbolicValue) -> Z3Expression {
        let lhs = self.integer_variable(path);
        let rhs = self.get_symbolic_as_z3_expression(value);
        unsafe { z3_sys::Z3_mk_eq(self.z3_context, lhs, rhs) }
    }

    pub fn convert_to_bool_sort(&self, exp: Z3Expression) -> Z3Expression {
        unsafe {
            let sort = z3_sys::Z3_get_sort(self.z3_context, exp);
            let sort_kind = z3_sys::Z3_get_sort_kind(self.z3_context, sort);
            if sort_kind != z3_sys::SortKind::Bool {
                let if_cond = z3_sys::Z3_mk_eq(self.z3_context, exp, self.zero);
                let true_ast = z3_sys::Z3_mk_true(self.z3_context);
                let false_ast = z3_sys::Z3_mk_false(self.z3_context);
                self.make_ite_z3_expression(if_cond, false_ast, true_ast)
            } else {
                exp
            }
        }
    }

    /// Convert a symbolic value into Z3 AST
    /// The AST's sort might be `int` or `bool`
    pub fn get_symbolic_as_z3_expression(&self, symbolic_value: &SymbolicValue) -> Z3Expression {
        use Expression::*;
        match &symbolic_value.expression {
            Drop(..) => unimplemented!(),

            Numerical(path) => self.integer_variable(path),

            Top | Bottom => self.make_constant(&ConstantValue::Top),

            And { left, right } => {
                let left_ast = self.get_symbolic_as_z3_expression(left);
                let right_ast = self.get_symbolic_as_z3_expression(right);
                unsafe { z3_sys::Z3_mk_and(self.z3_context, 2, vec![left_ast, right_ast].as_ptr()) }
            }

            CompileTimeConstant(const_value) => self.make_constant(const_value),

            Equals { left, right } => self.make_comparison(left, right, z3_sys::Z3_mk_eq),

            GreaterOrEqual { left, right } => self.make_comparison(left, right, z3_sys::Z3_mk_ge),

            GreaterThan { left, right } => self.make_comparison(left, right, z3_sys::Z3_mk_gt),

            LessOrEqual { left, right } => self.make_comparison(left, right, z3_sys::Z3_mk_le),

            LessThan { left, right } => self.make_comparison(left, right, z3_sys::Z3_mk_lt),

            LogicalNot { operand } => {
                self.make_not_z3_expression(self.get_symbolic_as_z3_expression(operand))
            }

            Ne { left, right } => {
                self.make_not_z3_expression(self.make_comparison(left, right, z3_sys::Z3_mk_eq))
            }

            Or { left, right } => {
                let left_ast = self.get_symbolic_as_z3_expression(left);
                let right_ast = self.get_symbolic_as_z3_expression(right);
                unsafe { z3_sys::Z3_mk_or(self.z3_context, 2, vec![left_ast, right_ast].as_ptr()) }
            }

            Variable { path, var_type } => self.typed_variable(path, var_type),

            Cast { .. }
            | HeapBlock { .. }
            | Join { .. }
            // | Offset { .. }
            | Reference(..)
            | Widen { .. } => {
                unimplemented!("Cannot cast symbolic value into Z3 ast");
            }
        }
    }

    /// Clean the state of the solver
    pub fn reset(&self) {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            z3_sys::Z3_solver_reset(self.z3_context, self.z3_solver);
        }
    }

    // The following are private

    /// Create a integer variable without type information
    /// Because Apron does not preserve type information
    fn integer_variable(&self, path: &Rc<Path>) -> Z3Expression {
        let path_str = CString::new(format!("{:?}", path)).unwrap();
        unsafe {
            let path_symbol = z3_sys::Z3_mk_string_symbol(self.z3_context, path_str.into_raw());
            let sort = self.int_sort;
            z3_sys::Z3_mk_const(self.z3_context, path_symbol, sort)
        }
    }

    fn typed_variable(&self, path: &Rc<Path>, var_type: &ExpressionType) -> Z3Expression {
        let path_str = CString::new(format!("{:?}", path)).unwrap();
        unsafe {
            let path_symbol = z3_sys::Z3_mk_string_symbol(self.z3_context, path_str.into_raw());
            let sort = self.get_sort_for(var_type);
            z3_sys::Z3_mk_const(self.z3_context, path_symbol, sort)
        }
    }

    fn get_sort_for(&self, var_type: &ExpressionType) -> z3_sys::Z3_sort {
        use self::ExpressionType::*;
        match var_type {
            Bool => self.bool_sort,
            I8 | I16 | I32 | I64 | I128 | Isize | U8 | U16 | U32 | U64 | U128 | Usize => {
                self.int_sort
            }
            NonPrimitive | Reference => self.any_sort,
        }
    }

    fn make_constant(&self, constant: &ConstantValue) -> Z3Expression {
        use ConstantValue::*;
        match constant {
            Bottom => unsafe {
                z3_sys::Z3_mk_fresh_const(self.z3_context, self.empty_str, self.any_sort)
            },
            Top => unsafe {
                z3_sys::Z3_mk_fresh_const(self.z3_context, self.empty_str, self.any_sort)
            },
            Function(..) => unsafe {
                z3_sys::Z3_mk_fresh_const(self.z3_context, self.empty_str, self.any_sort)
            },
            Int(integer) => self.make_constant_integer(integer),
        }
    }

    fn make_comparison(
        &self,
        lhs: &Rc<SymbolicValue>,
        rhs: &Rc<SymbolicValue>,
        op: unsafe extern "C" fn(
            ctx: z3_sys::Z3_context,
            lhs: Z3Expression,
            rhs: Z3Expression,
        ) -> Z3Expression,
    ) -> Z3Expression {
        let left = self.get_symbolic_as_z3_expression(lhs);
        let right = self.get_symbolic_as_z3_expression(rhs);
        unsafe { op(self.z3_context, left, right) }
    }

    fn set_backtrack_position(&self) {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            z3_sys::Z3_solver_push(self.z3_context, self.z3_solver);
        }
    }

    fn backtrack(&self) {
        unsafe {
            let _guard = Z3_MUTEX.lock().unwrap();
            z3_sys::Z3_solver_pop(self.z3_context, self.z3_solver, 1);
        }
    }

    fn get_symbol_for(&self, path: &Rc<Path>) -> z3_sys::Z3_symbol {
        let path_str = CString::new(format!("{:?}", path)).unwrap();
        unsafe { z3_sys::Z3_mk_string_symbol(self.z3_context, path_str.into_raw()) }
    }

    fn make_constant_integer(&self, integer: &Integer) -> Z3Expression {
        if let Some(integer_i64) = integer.to_i64() {
            unsafe { z3_sys::Z3_mk_int64(self.z3_context, integer_i64, self.int_sort) }
        } else {
            let num_str = format!("{}", *integer);
            let c_string = CString::new(num_str).unwrap();
            unsafe { z3_sys::Z3_mk_numeral(self.z3_context, c_string.into_raw(), self.int_sort) }
        }
    }

    fn expr2int(&self, expr: &LinearExpression) -> Z3Expression {
        let mut res = unsafe { z3_sys::Z3_mk_int64(self.z3_context, 0, self.int_sort) };
        for (var, coff) in expr {
            let path_symbol = self.get_symbol_for(var);
            let v = unsafe { z3_sys::Z3_mk_const(self.z3_context, path_symbol, self.int_sort) };
            let c = self.make_constant_integer(coff);
            // Compute multiplication
            let tmp = vec![c, v];
            let term = unsafe { z3_sys::Z3_mk_mul(self.z3_context, 2, tmp.as_ptr()) };
            // Compute addition
            let tmp = vec![res, term];
            res = unsafe { z3_sys::Z3_mk_add(self.z3_context, 2, tmp.as_ptr()) };
            // res = Int::add(self.ctx(), &[&res, &term]);
        }
        let constant = self.make_constant_integer(&expr.constant());
        // Compute addition
        // Int::add(self.ctx(), &[&res, &constant])
        let tmp = vec![res, constant];
        unsafe { z3_sys::Z3_mk_add(self.z3_context, 2, tmp.as_ptr()) }
    }
}

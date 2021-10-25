use crate::analysis::memory::constant_value::ConstantValue;
use crate::analysis::memory::expression::Expression;
use crate::analysis::memory::path::Path;
use crate::analysis::memory::symbolic_domain::SymbolicDomain;
use crate::analysis::memory::symbolic_value::{SymbolicValue, SymbolicValueTrait};
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, GetManagerTrait,
};
use crate::analysis::numerical::lattice::LatticeTrait;
use rug::Integer;
use rustc_middle::mir;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    // Only stores the values of paths that are integers
    pub numerical_domain: ApronAbstractDomain<DomainType>,
    // Stores all the symbolic values
    pub symbolic_domain: SymbolicDomain,
    // Stores branch conditions
    pub exit_conditions: HashMap<mir::BasicBlock, Rc<SymbolicValue>>,
}

impl<DomainType> fmt::Debug for AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "numerical: {:?}, symbolic: {:?}",
            self.numerical_domain, self.symbolic_domain
        )
    }
}

impl<DomainType: ApronDomainType> AbstractDomain<DomainType>
where
    DomainType: ApronDomainType,
    ApronAbstractDomain<DomainType>: GetManagerTrait,
{
    // A bottom domain means the basic block is unreachable
    pub fn is_bottom(&self) -> bool {
        self.numerical_domain.is_bottom()
    }

    pub fn is_top(&self) -> bool {
        self.symbolic_domain.is_top() && self.numerical_domain.is_top()
    }

    pub fn leq(&self, other: &Self) -> bool {
        // TODO: consider symbolic domain
        self.numerical_domain.leq(&other.numerical_domain)
    }

    pub fn is_empty(&self) -> bool {
        self.numerical_domain.is_top() && self.symbolic_domain.size() == 0
    }

    pub fn default() -> Self {
        Self {
            numerical_domain: ApronAbstractDomain::default(),
            symbolic_domain: SymbolicDomain::default(),
            exit_conditions: HashMap::new(),
        }
    }

    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        use itertools::Itertools;
        let n = self.numerical_domain.get_paths_iter();
        let s = self.symbolic_domain.get_paths_iter();
        n.iter().merge(s.iter()).unique().cloned().collect()
    }

    pub fn remove(&mut self, path: &Rc<Path>) {
        self.symbolic_domain.forget(path);
        self.numerical_domain.forget(path);
    }

    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        debug!("Renaming {:?} to {:?}", old_path, new_path);
        self.numerical_domain.rename(old_path, new_path);
        self.symbolic_domain.rename(old_path, new_path);
    }

    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        self.numerical_domain.duplicate(old_path, new_path);
        self.symbolic_domain.duplicate(old_path, new_path);
    }

    /// Returns a reference to the value associated with the given path, if there is one.
    pub fn value_at(&self, path: &Rc<Path>) -> Option<Rc<SymbolicValue>> {
        // self.symbolic_domain.value_map.get(path).map(|x| x.clone())
        if let Some(sym_value) = self.symbolic_domain.value_map.get(path) {
            Some(sym_value.clone())
        } else if self.numerical_domain.contains(path) {
            let interval = self.numerical_domain.get_interval(path);
            if let Ok(const_int) = Integer::try_from(interval) {
                Some(SymbolicValue::make_from(
                    Expression::CompileTimeConstant(ConstantValue::Int(const_int)),
                    1,
                ))
            } else {
                let e = Expression::Numerical(path.clone());
                Some(SymbolicValue::make_from(e, 1))
            }
        } else {
            None
        }
    }

    /// Updates the path to value map so that the given path now points to the given value.
    pub fn update_value_at(&mut self, path: Rc<Path>, value: Rc<SymbolicValue>) {
        debug!("Updating value at {:?}, value: {:?}", path, value);
        if value.is_bottom() || value.is_top() {
            debug!("Value is bottom or top, ignore");
            self.symbolic_domain.value_map.remove(&path);
            self.numerical_domain.forget(&path);
            return;
        }

        // Handle numerical values, store them in numerical domain
        // Case 1: value is already in numerical domain, so there is only a path in expression
        if let Expression::Numerical(rpath) = &value.expression {
            debug!("Value is numerical, store in numerical domain");
            self.numerical_domain.assign_var(path, rpath.clone());
        }
        // Case 2: value is a compile time constant, and is of type integer
        else if let Expression::CompileTimeConstant(constant_domain) = &value.expression {
            if let Some(integer) = constant_domain.try_get_integer() {
                debug!("Value is constant integer, store in numerical domain");
                self.numerical_domain.assign_int(path, integer);
            } else {
                debug!("Value is constant but not integer, store in symbolic domain");
                self.symbolic_domain.value_map.insert(path, value.clone());
            }
        }
        // Case 3: value is a variable of type integer
        else if let Expression::Variable {
            path: rpath,
            var_type,
        } = &value.expression
        {
            if var_type.is_integer() {
                debug!("Value is integer variable, store in both numerical and symbolic domain");
                self.numerical_domain
                    .assign_var(path.clone(), rpath.clone());
                self.symbolic_domain.value_map.insert(path, value.clone());
            } else {
                debug!("Value is a variable but not integer store in symbolic domain");
                self.symbolic_domain.value_map.insert(path, value.clone());
            }
        } else {
            // Reach here if value is not numerical, store them in symbolic domain
            debug!("Value is not numerical, store in symbolic domain");
            self.symbolic_domain.value_map.insert(path, value.clone());
        }
    }

    pub fn join(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.join(&other.numerical_domain);
        let symbolic = self.symbolic_domain.lub(&other.symbolic_domain);
        Self {
            numerical_domain: numerical,
            symbolic_domain: symbolic,
            exit_conditions: HashMap::new(),
        }
    }

    // TODO: implement meet for symbolic domain
    pub fn meet(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.meet(&other.numerical_domain);
        Self {
            numerical_domain: numerical,
            symbolic_domain: other.symbolic_domain.clone(),
            exit_conditions: HashMap::new(),
        }
    }

    pub fn widening_with(&self, other: &Self) -> Self {
        let numerical = self.numerical_domain.widening_with(&other.numerical_domain);
        let symbolic = self.symbolic_domain.widening_with(&other.symbolic_domain);

        Self {
            numerical_domain: numerical,
            symbolic_domain: symbolic,
            exit_conditions: HashMap::new(),
        }
    }

    // TODO: implement narrowing for numerical domain and symbolic domain
    pub fn narrowing_with(&self, other: &Self) -> Self {
        let numerical = self
            .numerical_domain
            .narrowing_with(&other.numerical_domain);

        Self {
            numerical_domain: numerical,
            // Seems like no need to do narrowing for symbolic domain
            symbolic_domain: other.symbolic_domain.clone(),
            exit_conditions: HashMap::new(),
        }
    }

    pub fn subset(&self, other: &Self) -> bool {
        let value_map1 = &self.symbolic_domain.value_map;
        let value_map2 = &other.symbolic_domain.value_map;
        if value_map1.len() > value_map2.len() {
            return false;
        }
        for (path, val1) in value_map1.iter() {
            match value_map2.get(path) {
                Some(val2) => {
                    if !(val1.subset(val2)) {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        true
    }
}

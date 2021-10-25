use crate::analysis::memory::expression::Expression;
use crate::analysis::memory::path::Path;
use crate::analysis::memory::symbolic_value::{self, SymbolicValue, SymbolicValueTrait};
use crate::analysis::numerical::lattice::LatticeTrait;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub struct SymbolicDomain {
    is_top: bool,
    is_bottom: bool,
    pub value_map: HashMap<Rc<Path>, Rc<SymbolicValue>>,
}

impl SymbolicDomain {
    pub fn get_paths_iter(&self) -> Vec<Rc<Path>> {
        self.value_map.keys().cloned().collect()
    }

    pub fn default() -> Self {
        Self {
            is_top: false,
            is_bottom: true,
            value_map: HashMap::new(),
        }
    }

    pub fn rename(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if let Some(value) = self.value_map.get(old_path) {
            let value = value.clone();
            self.value_map.remove(old_path);
            self.value_map.insert(new_path.clone(), value);
        }
    }

    pub fn duplicate(&mut self, old_path: &Rc<Path>, new_path: &Rc<Path>) {
        if let Some(value) = self.value_map.get(old_path) {
            let value = value.clone();
            self.value_map.insert(new_path.clone(), value);
        }
    }

    pub fn contains(&self, path: &Rc<Path>) -> bool {
        self.value_map.contains_key(path)
    }

    pub fn depend_on(&self, path: &Rc<Path>) -> bool {
        self.value_map
            .iter()
            .any(|(_, symbolic_value)| match &symbolic_value.expression {
                Expression::Numerical(p)
                | Expression::Reference(p)
                | Expression::Variable { path: p, .. } => path == p || p.is_rooted_by(path),
                _ => {
                    let value = self
                        .value_map
                        .get(path)
                        .map(|x| x.clone())
                        .unwrap_or(symbolic_value::BOTTOM.into());
                    symbolic_value.depend_on_path_value(path, &value)
                }
            })
    }

    pub fn size(&self) -> usize {
        self.value_map.len()
    }

    pub fn forget(&mut self, path: &Rc<Path>) {
        self.value_map.remove(path);
    }
}

impl fmt::Debug for SymbolicDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value_map)
    }
}

impl LatticeTrait for SymbolicDomain {
    fn top() -> Self {
        Self {
            is_top: true,
            is_bottom: false,
            value_map: HashMap::new(),
        }
    }

    fn is_top(&self) -> bool {
        if self.is_top {
            true
        } else {
            self.value_map.iter().all(|(_k, v)| v.is_top())
        }
    }

    fn set_to_top(&mut self) {
        self.value_map.clear();
        self.is_top = true;
        self.is_bottom = false;
    }

    fn bottom() -> Self {
        Self {
            is_top: false,
            is_bottom: true,
            value_map: HashMap::new(),
        }
    }

    fn is_bottom(&self) -> bool {
        if self.is_bottom {
            true
        } else {
            self.value_map.iter().all(|(_k, v)| v.is_bottom())
        }
    }

    fn set_to_bottom(&mut self) {
        self.value_map.clear();
        self.is_bottom = true;
        self.is_top = false;
    }

    fn lub(&self, other: &Self) -> Self {
        let value_map1 = &self.value_map;
        let mut result = other.value_map.clone();
        for (path, val1) in value_map1.iter() {
            match other.value_map.get(path) {
                Some(val2) => {
                    result.insert(path.clone(), val1.join(val2.clone()));
                }
                None => {
                    result.insert(path.clone(), val1.clone());
                }
            }
        }

        Self {
            value_map: result,
            is_top: false,
            is_bottom: false,
        }
    }

    /// Widening for symbolic domain
    fn widening_with(&self, other: &Self) -> Self {
        let value_map1 = &self.value_map;
        let mut result = other.value_map.clone();
        for (path, val1) in value_map1.iter() {
            match other.value_map.get(path) {
                Some(val2) => {
                    if val1.is_widened() {
                        result.insert(path.clone(), val1.clone());
                    } else if val2.is_widened() {
                        result.insert(path.clone(), val2.clone());
                    } else {
                        let res = val1.join(val2.clone()).widen(path);
                        result.insert(path.clone(), res);
                    }
                }
                None => {
                    result.insert(path.clone(), val1.clone());
                }
            }
        }

        Self {
            value_map: result,
            is_top: false,
            is_bottom: false,
        }
    }
}

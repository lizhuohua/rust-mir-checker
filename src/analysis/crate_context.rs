// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::analysis::memory::constant_value::ConstantValueCache;
use crate::analysis::memory::known_names::KnownNamesCache;
use rustc_errors::DiagnosticBuilder;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::subst::SubstsRef;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};

/// A visitor that takes information gathered by the Rust compiler when compiling a particular
/// crate and then analyses some of the functions in that crate to see if any of the assertions
/// and implicit assertions in the MIR bodies might be false and generates warning for those.
///
pub struct CrateContext<'compiler, 'tcx> {
    /// Stores the diagnostic messages for the current crate
    pub buffered_diagnostics: Vec<DiagnosticBuilder<'compiler>>,

    /// Caches the constant in the current crate
    pub constant_value_cache: ConstantValueCache<'tcx>,

    /// Caches the name of each function in the current crate
    pub function_name_cache: HashMap<DefId, String>,

    pub known_names_cache: KnownNamesCache,

    pub substs_cache: HashMap<DefId, SubstsRef<'tcx>>,
}

impl<'compiler, 'tcx> Debug for CrateContext<'compiler, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        "CrateContext".fmt(f)
    }
}

impl<'compiler, 'tcx> Default for CrateContext<'compiler, 'tcx> {
    fn default() -> Self {
        CrateContext {
            buffered_diagnostics: Vec::new(),
            constant_value_cache: ConstantValueCache::default(),
            known_names_cache: KnownNamesCache::create_cache_from_language_items(),
            function_name_cache: HashMap::new(),
            substs_cache: HashMap::new(),
        }
    }
}

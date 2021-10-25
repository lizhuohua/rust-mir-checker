// This file is adapted from MIRAI (https://github.com/facebookexperimental/MIRAI)
// Original author: Herman Venter <hermanv@fb.com>
// Original copyright header:

// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use rustc_hir::def_id::DefId;
use rustc_hir::definitions::{DefPathData, DisambiguatedDefPathData};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

/// Well known definitions (language provided items) that are treated in special ways.
#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Hash, Ord)]
pub enum KnownNames {
    /// This is not a known name
    None,
    MirCheckerVerify,
    RustAlloc,
    RustAllocZeroed,
    RustDealloc,
    RustRealloc,
    StdMemSizeOf,
    StdPanickingBeginPanic,
    StdPanickingBeginPanicFmt,

    StdIntoVec,
    CoreOpsIndex,
    StdFrom,
    StdAsMutPtr,

    VecFromRawParts,
}

/// An analysis lifetime cache that contains a map from def ids to known names.
pub struct KnownNamesCache {
    name_cache: HashMap<DefId, KnownNames>,
}

type Iter<'a> = std::slice::Iter<'a, rustc_hir::definitions::DisambiguatedDefPathData>;

impl KnownNamesCache {
    /// Create an empty known names cache.
    /// This cache is re-used by every successive MIR visitor instance.
    pub fn create_cache_from_language_items() -> KnownNamesCache {
        let name_cache = HashMap::new();
        KnownNamesCache { name_cache }
    }

    /// Get the well known name for the given def id and cache the association.
    /// I.e. the first call for an unknown def id will be somewhat costly but
    /// subsequent calls will be cheap. If the def_id does not have an actual well
    /// known name, this returns KnownNames::None.
    pub fn get(&mut self, tcx: TyCtxt<'_>, def_id: DefId) -> KnownNames {
        *self
            .name_cache
            .entry(def_id)
            .or_insert_with(|| Self::get_known_name_for(tcx, def_id))
    }

    /// Uses information obtained from tcx to figure out which well known name (if any)
    /// this def id corresponds to.
    fn get_known_name_for(tcx: TyCtxt<'_>, def_id: DefId) -> KnownNames {
        use std::ops::Deref;
        use DefPathData::*;

        // E.g. DefPath { data: [DisambiguatedDefPathData { data: TypeNs("mem"), disambiguator: 0 }, DisambiguatedDefPathData { data: ValueNs("size_of"), disambiguator: 0 }], krate: crate2 }
        let def_path = &tcx.def_path(def_id);
        debug!("TEST: {:?}", def_path);
        let def_path_data_iter = def_path.data.iter();

        // helper to get next elem from def path and return its name, if it has one
        let get_path_data_elem_name =
            |def_path_data_elem: Option<&rustc_hir::definitions::DisambiguatedDefPathData>| {
                def_path_data_elem.and_then(|ref elem| {
                    let DisambiguatedDefPathData { data, .. } = elem;
                    // Get only something in the type/value namespace, and ignore others
                    match &data {
                        TypeNs(name) | ValueNs(name) => Some(*name),
                        _ => None,
                    }
                })
            };

        let get_known_name_for_alloc_namespace =
            |mut def_path_data_iter: Iter<'_>| match get_path_data_elem_name(
                def_path_data_iter.next(),
            ) {
                Some(n) if n.as_str().deref() == "" => {
                    get_path_data_elem_name(def_path_data_iter.next())
                        .map(|n| match n.as_str().deref() {
                            "__rust_alloc" => KnownNames::RustAlloc,
                            "__rust_alloc_zeroed" => KnownNames::RustAllocZeroed,
                            "__rust_dealloc" => KnownNames::RustDealloc,
                            "__rust_realloc" => KnownNames::RustRealloc,
                            _ => KnownNames::None,
                        })
                        .unwrap_or(KnownNames::None)
                }
                _ => KnownNames::None,
            };

        let get_known_name_for_mem_namespace = |mut def_path_data_iter: Iter<'_>| {
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "size_of" => KnownNames::StdMemSizeOf,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_ops_namespace = |mut def_path_data_iter: Iter<'_>| {
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "index" | "index_mut" => KnownNames::CoreOpsIndex,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_panicking_namespace = |mut def_path_data_iter: Iter<'_>| {
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "begin_panic" | "panic" => KnownNames::StdPanickingBeginPanic,
                    "begin_panic_fmt" | "panic_fmt" => KnownNames::StdPanickingBeginPanicFmt,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_slice_namespace = |mut def_path_data_iter: Iter<'_>| {
            def_path_data_iter.next();
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "into_vec" => KnownNames::StdIntoVec,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_convert_namespace = |mut def_path_data_iter: Iter<'_>| {
            def_path_data_iter.next();
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "from" => KnownNames::StdFrom,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_vec_namespace = |mut def_path_data_iter: Iter<'_>| {
            def_path_data_iter.next();
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "as_mut_ptr" => KnownNames::StdAsMutPtr,
                    "from_raw_parts" => KnownNames::VecFromRawParts,
                    _ => KnownNames::None,
                })
                .unwrap_or(KnownNames::None)
        };

        let get_known_name_for_known_crate = |mut def_path_data_iter: Iter<'_>| {
            get_path_data_elem_name(def_path_data_iter.next())
                .map(|n| match n.as_str().deref() {
                    "alloc" => get_known_name_for_alloc_namespace(def_path_data_iter),
                    "mem" => get_known_name_for_mem_namespace(def_path_data_iter),
                    "ops" => get_known_name_for_ops_namespace(def_path_data_iter),
                    "slice" => get_known_name_for_slice_namespace(def_path_data_iter),
                    "panicking" => get_known_name_for_panicking_namespace(def_path_data_iter),
                    "convert" => get_known_name_for_convert_namespace(def_path_data_iter),
                    "vec" => get_known_name_for_vec_namespace(def_path_data_iter),
                    "mir_checker_verify" => KnownNames::MirCheckerVerify,
                    _ => {
                        debug!("Normal function: {:?}", n.as_str());
                        KnownNames::None
                    }
                })
                .unwrap_or(KnownNames::None)
        };

        let crate_name = tcx.crate_name(def_id.krate);
        match crate_name.as_str().deref() {
            // Only recognize known functions from the following crates
            "alloc" | "core" | "macros" | "std" => {
                get_known_name_for_known_crate(def_path_data_iter)
            }
            _ => KnownNames::None,
        }
    }
}

use crate::analysis::diagnostics::DiagnosticsForDefId;
use crate::analysis::memory::symbolic_value::SymbolicValue;
use crate::analysis::option::AnalysisOption;
use crate::analysis::wto::Wto;
use log::{debug, info};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

/// Cache the wto so we do not need to recompute them when analyzing a function multiple times
pub struct WtoCache<'tcx> {
    value: HashMap<DefId, Wto<'tcx>>,
}

impl<'tcx> WtoCache<'tcx> {
    pub fn get(&self, def_id: DefId) -> Option<&Wto<'tcx>> {
        self.value.get(&def_id)
    }

    pub fn insert(&mut self, def_id: DefId, wto: Wto<'tcx>) {
        self.value.insert(def_id, wto);
    }
}

impl<'tcx> Default for WtoCache<'tcx> {
    fn default() -> Self {
        Self {
            value: HashMap::new(),
        }
    }
}

/// Stores the global information of the analysis
pub struct GlobalContext<'tcx, 'compiler> {
    /// The central data structure of the compiler
    pub tcx: TyCtxt<'tcx>,

    /// Represents the data associated with a compilation session for a single crate
    pub session: &'compiler Session,

    /// The entry function of the analysis
    pub entry_point: DefId,

    /// Stores the DefIds that have been already checked, to avoid redundant check
    pub checked_def_ids: HashSet<DefId>,

    /// Stores the Heaps that have been already dropped, to detect double-free, use-after-free, etc.
    pub dropped_heaps: HashSet<Rc<SymbolicValue>>,

    /// Cache for the Weak Topological Ordering
    pub wto_cache: WtoCache<'tcx>,

    /// Cache for the name of each DefId
    pub function_name_cache: HashMap<DefId, Rc<String>>,

    /// Customized options that may change the behavior of the analysis
    pub analysis_options: AnalysisOption,

    /// Generated diagnostic messages for each DefId
    pub diagnostics_for: DiagnosticsForDefId<'compiler>,
}

impl<'tcx, 'compiler> fmt::Debug for GlobalContext<'tcx, 'compiler> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlobalContext")
    }
}

impl<'tcx, 'compiler> GlobalContext<'tcx, 'compiler> {
    pub fn new(
        session: &'compiler Session,
        tcx: TyCtxt<'tcx>,
        analysis_options: AnalysisOption,
    ) -> Option<Self> {
        if analysis_options.show_entries {
            let mut names = HashSet::new();
            for def_id in tcx.body_owners() {
                if tcx.def_kind(def_id) == DefKind::Fn || tcx.def_kind(def_id) == DefKind::AssocFn {
                    let name = tcx.item_name(def_id.to_def_id());
                    if !names.contains(&name) {
                        names.insert(name);
                        println!("{}", name);
                    }
                    // println!("{}", def_id.to_def_id().index.as_u32());
                }
            }
            return None;
        }

        if analysis_options.show_entries_index {
            // let mut names = HashSet::new();
            for def_id in tcx.body_owners() {
                if tcx.def_kind(def_id) == DefKind::Fn || tcx.def_kind(def_id) == DefKind::AssocFn {
                    // let name = tcx.item_name(def_id.to_def_id());
                    // if !names.contains(&name) {
                    //     names.insert(name);
                    //     println!("{}", name);
                    // }
                    println!("{}", def_id.to_def_id().index.as_u32());
                }
            }
            return None;
        }

        info!("Initializing GlobalContext");
        let mut entry_func = None;

        // List functions
        for def_id in tcx.body_owners() {
            let def_kind = tcx.def_kind(def_id);
            // Find the DefId for the entry point, note that the entry point must be a function
            if def_kind == DefKind::Fn || def_kind == DefKind::AssocFn {
                // If `entry_def_id_index` flag is provided, find entry point according to the index
                if let Some(entry_def_id_index) = analysis_options.entry_def_id_index {
                    let item_name = tcx.item_name(def_id.to_def_id());
                    if def_id.to_def_id().index.as_u32() == entry_def_id_index {
                        entry_func = Some(def_id);
                        debug!("Entry Point: {:?}, DefId: {:?}", item_name, def_id);
                    } else {
                        debug!(
                            "Name: {:?}, DefId: {:?}, DefKind: {:?}",
                            tcx.item_name(def_id.to_def_id()),
                            def_id,
                            def_kind
                        );
                    }
                }
                // If not, find entry point according to the function name
                else {
                    let entry_point = analysis_options.entry_point.clone();
                    let item_name = tcx.item_name(def_id.to_def_id());
                    if item_name.to_string() == *entry_point {
                        entry_func = Some(def_id);
                        debug!("Entry Point: {:?}, DefId: {:?}", item_name, def_id);
                    } else {
                        debug!(
                            "Name: {:?}, DefId: {:?}, DefKind: {:?}",
                            tcx.item_name(def_id.to_def_id()),
                            def_id,
                            def_kind
                        );
                    }
                }
            }
        }

        if let Some(entry) = entry_func {
            Some(Self {
                tcx,
                session,
                function_name_cache: HashMap::new(),
                entry_point: entry.to_def_id(),
                checked_def_ids: HashSet::new(),
                dropped_heaps: HashSet::new(),
                wto_cache: WtoCache::default(),
                analysis_options,
                diagnostics_for: DiagnosticsForDefId::default(),
            })
        } else {
            error!("Entry point not found");
            None
        }
    }

    pub fn get_wto(&mut self, def_id: DefId) -> Wto<'tcx> {
        let mir = self.tcx.optimized_mir(def_id);
        let wto;
        // First see whether the wto has been already computed
        if let Some(cached_wto) = self.wto_cache.get(def_id) {
            debug!("Using cached w.t.o for {}", self.tcx.item_name(def_id));
            wto = cached_wto.clone();
        } else {
            // If not, compute the wto
            wto = Wto::new(mir);
            debug!(
                "Compute the new w.t.o for {}: {:?}",
                self.tcx.item_name(def_id),
                wto
            );
            // Cache the wto
            self.wto_cache.insert(def_id, wto.clone());
        }
        wto
    }
}

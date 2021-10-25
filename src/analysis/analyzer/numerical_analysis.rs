use crate::analysis::abstract_domain::AbstractDomain;
use crate::analysis::analysis_result::{AnalysisInfo, Result};
use crate::analysis::analyzer::analysis_trait::StaticAnalysis;
use crate::analysis::diagnostics::Diagnostic;
use crate::analysis::global_context::GlobalContext;
use crate::analysis::mir_visitor::body_visitor::WtoFixPointIterator;
use crate::analysis::numerical::apron_domain::{
    ApronAbstractDomain, ApronDomainType, ApronInterval, ApronLinearEqualities, ApronOctagon,
    ApronPkgridPolyhedraLinCongruences, ApronPolyhedra, ApronPplLinearCongruences,
    ApronPplPolyhedra, GetManagerTrait,
};
use crate::analysis::option::AbstractDomainType;
use log::info;
use rustc_hir::def_id::DefId;
use std::time::Instant;

/// Traverse over a crate, analyze all functions and emit diagnoses
pub struct NumericalAnalysis<'tcx, 'a, 'compiler> {
    /// The global context
    pub context: &'a mut GlobalContext<'tcx, 'compiler>,
}

impl<'tcx, 'a, 'compiler> StaticAnalysis<'tcx, 'a, 'compiler>
    for NumericalAnalysis<'tcx, 'a, 'compiler>
{
    fn new(context: &'a mut GlobalContext<'tcx, 'compiler>) -> Self {
        NumericalAnalysis { context }
    }

    fn emit_diagnostics(&mut self) {
        let mut diagnostics: Vec<&mut Diagnostic<'_>> = self
            .context
            .diagnostics_for
            .map
            .values_mut()
            .flatten()
            .collect();

        diagnostics.sort_by(Diagnostic::compare);

        // If `deny_warnings` flag is set, change all diagnoses' level to `error`
        // This is used for debugging
        if self.context.analysis_options.deny_warnings {
            for diag in &mut diagnostics {
                diag.builder.level = rustc_errors::Level::Error;
            }
        }

        // According to `suppress_warnings` flag, filter out warnings that users want to ignore
        let mut diagnostics: Vec<&mut Diagnostic<'_>> =
            if let Some(suppressed_warnings) = &self.context.analysis_options.suppressed_warnings {
                let mut res: Vec<&mut Diagnostic<'_>> = Vec::new();
                for diag in diagnostics.iter_mut() {
                    if suppressed_warnings.contains(&diag.cause) {
                        diag.cancel();
                    } else {
                        res.push(diag);
                    }
                }
                res
            } else {
                diagnostics.into_iter().collect()
            };

        // According to `memory_safety_only` flag, filter only memory-safety diagnosis
        // Cancel other diagnoses that will not be emitted
        let diagnostics_to_emit: Vec<&mut Diagnostic<'_>> =
            if self.context.analysis_options.memory_safety_only {
                let mut res: Vec<&mut Diagnostic<'_>> = Vec::new();
                for diag in diagnostics.iter_mut() {
                    if diag.is_memory_safety {
                        res.push(diag);
                    } else {
                        diag.cancel();
                    }
                }
                res
            } else {
                diagnostics.into_iter().collect()
            };

        fn emit(db: &mut Diagnostic<'_>) {
            db.emit();
        }

        diagnostics_to_emit.into_iter().for_each(emit);
    }

    fn run(&mut self) -> Result<AnalysisInfo> {
        let timer = Instant::now();

        info!("================== Numerical Analysis Starts ==================");
        info!(
            "Abstract Domain Type: {:?}",
            self.context.analysis_options.domain_type
        );
        info!(
            "Widening Delay: {}",
            self.context.analysis_options.widening_delay
        );
        info!(
            "Start Analyzing Entry Point Function: {}",
            self.context.tcx.item_name(self.context.entry_point)
        );

        // Start analysis with the entry point
        let def_id = self.context.entry_point;

        match self.context.analysis_options.domain_type {
            AbstractDomainType::Interval => {
                self.analyze_function(def_id, AbstractDomain::<ApronInterval>::default());
            }
            AbstractDomainType::Octagon => {
                self.analyze_function(def_id, AbstractDomain::<ApronOctagon>::default());
            }
            AbstractDomainType::Polyhedra => {
                self.analyze_function(def_id, AbstractDomain::<ApronPolyhedra>::default());
            }
            AbstractDomainType::LinearEqualities => {
                self.analyze_function(def_id, AbstractDomain::<ApronLinearEqualities>::default());
            }
            AbstractDomainType::PplPolyhedra => {
                self.analyze_function(def_id, AbstractDomain::<ApronPplPolyhedra>::default());
            }
            AbstractDomainType::PplLinearCongruences => {
                self.analyze_function(
                    def_id,
                    AbstractDomain::<ApronPplLinearCongruences>::default(),
                );
            }
            AbstractDomainType::PkgridPolyhedraLinCongruences => {
                self.analyze_function(
                    def_id,
                    AbstractDomain::<ApronPkgridPolyhedraLinCongruences>::default(),
                );
            }
        }

        info!("================== Numerical Analysis Ends ==================");

        info!("================== Start To Output Diagnostics ==================");
        self.emit_diagnostics();

        Ok(AnalysisInfo {
            analysis_time: timer.elapsed(),
        })
    }

    fn analyze_function<DomainType>(
        &mut self,
        def_id: DefId,
        abstract_domain: AbstractDomain<DomainType>,
    ) where
        DomainType: ApronDomainType,
        ApronAbstractDomain<DomainType>: GetManagerTrait,
    {
        let func_name = self.context.tcx.item_name(def_id);
        info!(
            "================== Fixed-Point Algorithm Starts To Analyze: {} ==================",
            func_name
        );

        // Compute the fixed-point of the function specified by `def_id`
        let mut wto_visitor =
            WtoFixPointIterator::new(self.context, def_id, abstract_domain, 0, vec![]);
        wto_visitor.init_promote_constants();
        wto_visitor.run();

        // Execute bug detector
        wto_visitor.run_checker();

        debug!(
            "{} diagnositcs for function {:?}",
            wto_visitor.buffered_diagnostics.len(),
            func_name
        );

        info!("================== Fixed-Point Algorithm Ends ==================");
    }
}

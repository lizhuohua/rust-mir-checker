use crate::analysis::diagnostics::DiagnosticCause;
use log::warn;

#[derive(Clone, Copy, Debug)]
pub enum AbstractDomainType {
    Interval,
    Octagon,
    Polyhedra,
    LinearEqualities,
    PplPolyhedra,
    PplLinearCongruences,
    PkgridPolyhedraLinCongruences,
}

#[derive(Clone, Debug)]
pub struct AnalysisOption {
    pub entry_point: String,
    pub entry_def_id_index: Option<u32>,
    pub domain_type: AbstractDomainType,
    pub widening_delay: u32,
    pub cleaning_delay: usize,
    pub narrowing_iteration: u32,
    pub show_entries: bool,
    pub show_entries_index: bool,
    pub deny_warnings: bool,
    pub memory_safety_only: bool,
    pub suppressed_warnings: Option<Vec<DiagnosticCause>>,
}

impl Default for AnalysisOption {
    fn default() -> Self {
        Self {
            entry_point: String::from("main"),
            entry_def_id_index: None,
            domain_type: AbstractDomainType::Interval,
            widening_delay: 5,
            cleaning_delay: 5,
            narrowing_iteration: 5,
            show_entries: false,
            show_entries_index: false,
            deny_warnings: false,
            memory_safety_only: false,
            suppressed_warnings: None,
        }
    }
}

impl AnalysisOption {
    pub fn from_args(args: &mut Vec<String>) -> Self {
        let mut indeices_to_remove = vec![];
        let mut res = Self::default();
        for (i, arg) in args.iter().enumerate() {
            if arg.starts_with("--") {
                match &arg[2..] {
                    "show_entries" => {
                        res.show_entries = true;
                        indeices_to_remove.push(i);
                    }
                    "show_entries_index" => {
                        res.show_entries_index = true;
                        indeices_to_remove.push(i);
                    }
                    "deny_warnings" => {
                        res.deny_warnings = true;
                        indeices_to_remove.push(i);
                    }
                    "memory_safety_only" => {
                        res.memory_safety_only = true;
                        indeices_to_remove.push(i);
                    }
                    "domain" => {
                        if let Some(domain_type) = Self::get_domain_type(&args[i + 1]) {
                            res.domain_type = domain_type;
                        } else {
                            warn!("Unknown domain type, use interval as default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "entry" => {
                        res.entry_point = args[i + 1].clone();
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "entry_def_id_index" => {
                        if let Ok(def_id_index) = args[i + 1].parse() {
                            res.entry_def_id_index = Some(def_id_index);
                        } else {
                            warn!("Invalid entry DefId index, use None as default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "widening_delay" => {
                        if let Ok(widening_delay) = args[i + 1].parse() {
                            res.widening_delay = widening_delay;
                        } else {
                            warn!("Invalid widening delay, use 5 as default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "narrowing_iteration" => {
                        if let Ok(narrowing_iteration) = args[i + 1].parse() {
                            res.narrowing_iteration = narrowing_iteration;
                        } else {
                            warn!("Invalid narrowing iteration, use 5 as default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "suppress_warnings" => {
                        if let Some(suppressed_warnings) =
                            Self::get_suppressed_warnings(&args[i + 1])
                        {
                            res.suppressed_warnings = Some(suppressed_warnings);
                        } else {
                            warn!("Invalid suppressed warning types, will not suppress any warnings by default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    "cleaning_delay" => {
                        if let Ok(cleaning_delay) = args[i + 1].parse() {
                            res.cleaning_delay = cleaning_delay;
                        } else {
                            warn!("Invalid cleaning delay, use 5 as default");
                        }
                        indeices_to_remove.push(i);
                        indeices_to_remove.push(i + 1);
                    }
                    _ => {}
                }
            }
        }
        indeices_to_remove.reverse();
        Self::remove_multiple(args, &indeices_to_remove);
        res
    }

    fn get_suppressed_warnings(arg: &str) -> Option<Vec<DiagnosticCause>> {
        let mut res = Vec::new();
        for ch in arg.chars() {
            match ch {
                'a' => res.push(DiagnosticCause::Arithmetic), // Arithmetic overflow
                'b' => res.push(DiagnosticCause::Bitwise),    // Bit-wise overflow
                's' => res.push(DiagnosticCause::Assembly),   // Inline assembly
                'c' => res.push(DiagnosticCause::Comparison), // Comparison operations
                'd' => res.push(DiagnosticCause::DivZero), // Division by zero / remainder by zero
                'm' => res.push(DiagnosticCause::Memory),  // Memory-safety issues
                'p' => res.push(DiagnosticCause::Panic),   // Run into panic code
                'i' => res.push(DiagnosticCause::Index),   // Out-of-bounds access
                _ => return None,                          // Invalid flags
            }
        }
        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn get_domain_type(arg: &str) -> Option<AbstractDomainType> {
        match arg {
            "interval" => Some(AbstractDomainType::Interval),
            "octagon" => Some(AbstractDomainType::Octagon),
            "polyhedra" => Some(AbstractDomainType::Polyhedra),
            "linear_equalities" => Some(AbstractDomainType::LinearEqualities),
            "ppl_polyhedra" => Some(AbstractDomainType::PplPolyhedra),
            "ppl_linear_congruences" => Some(AbstractDomainType::PplLinearCongruences),
            "pkgrid_polyhedra_linear_congruences" => {
                Some(AbstractDomainType::PkgridPolyhedraLinCongruences)
            }
            _ => None,
        }
    }

    // Remove a list of indices from a vector
    // From https://stackoverflow.com/questions/57947441/remove-a-sequence-of-values-from-a-vec-in-rust
    fn remove_multiple<T>(source: &mut Vec<T>, indices_to_remove: &[usize]) -> Vec<T> {
        indices_to_remove
            .iter()
            .copied()
            .map(|i| source.swap_remove(i))
            .collect()
    }
}

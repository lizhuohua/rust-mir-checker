#![feature(rustc_private)]
#![feature(box_patterns)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

// Modules for static analyses
pub mod analysis {
    // Definitions of callbacks for rustc
    pub mod callback;
    // For error handling
    pub mod analysis_result;
    // The global state of the whole analysis process
    pub mod global_context;
    // The state of the crate analysis process
    pub mod crate_context;
    // Abstract domain
    pub mod abstract_domain;
    // Apron numerical domains
    pub mod numerical {
        pub mod apron_domain;
        pub mod interval;
        pub mod lattice;
        pub mod linear_constraint;
    }
    // Memory model
    pub mod memory {
        pub mod constant_value;
        pub mod expression;
        pub mod k_limits;
        pub mod known_names;
        pub mod path;
        pub mod symbolic_domain;
        pub mod symbolic_value;
        pub mod utils;
    }
    // Abstractly executes MIR statements
    pub mod mir_visitor {
        pub mod block_visitor;
        pub mod body_visitor;
        pub mod call_visitor;
        pub mod type_visitor;
    }
    // Different kinds of analyses
    pub mod analyzer {
        pub mod analysis_trait;
        pub mod numerical_analysis;
    }
    // Compute weak topological order (wto) and extract loop bounds
    pub mod wto;
    // Analysis options
    pub mod option;
    // SMT solver
    // pub mod smt;
    pub mod z3_solver;
    // The structure and helper functions for emitting diagnostics
    pub mod diagnostics;
}

// Modules for program property checkers
pub mod checker {
    pub mod assertion_checker;
    pub mod checker_trait;
}

// Useful utilities
pub mod utils;

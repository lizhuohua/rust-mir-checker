extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Used to generate `ap_version.h`, which is required for building Apron
    Command::new("./configure")
        .current_dir("apron/")
        .args(&["-no-ocaml", "-no-java"])
        .status()
        .unwrap();
    Command::new("make")
        .args(&["-C", "apron/apron/", "ap_version.h"])
        .status()
        .unwrap();

    // Compile external library apron
    // https://github.com/antoinemine/apron
    let mut builder = cc::Build::new()
        .include("apron/num")
        .include("apron/apron")
        .include("apron/itv")
        .include("apron/box")
        .include("apron/newpolka")
        .include("apron/ppl")
        .include("apron/products")
        .include("/usr/include")
        .flag("-std=c99")
        .flag("-U__STRICT_ANSI__")
        .flag("-fPIC")
        .flag("-O3")
        // .flag("-g")
        // .flag("-O0")
        .flag("-DNDEBUG")
        .flag("-Wcast-qual")
        .flag("-Wswitch")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wundef")
        .flag("-Wcast-align")
        .flag("-Wno-unused")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-function")
        .flag("-Werror-implicit-function-declaration")
        .flag("-Wbad-function-cast")
        .flag("-Wstrict-prototypes")
        .flag("-Wp,-w") // disable preprocessor warnings
        .clone();

    let mut cpp_builder = cc::Build::new()
        .cpp(true)
        .include("apron/apron")
        .include("apron/num")
        .include("apron/itv")
        .flag("-U__STRICT_ANSI__")
        .flag("-DNDEBUG")
        .flag("-O3")
        // .flag("-g")
        // .flag("-O0")
        .flag("-Wcast-qual")
        .flag("-Wswitch")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wundef")
        .flag("-Wcast-align")
        .flag("-Wno-unused")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-function")
        .flag("-fPIC")
        .flag("-Wp,-w") // disable preprocessor warnings
        .clone();

    // Apron MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        // .file("apron/apron/ap_linearize_aux.c")
        .file("apron/apron/ap_interval.c")
        .file("apron/apron/ap_scalar.c")
        .file("apron/apron/ap_linexpr0.c")
        .file("apron/apron/ap_linexpr1.c")
        .file("apron/apron/ap_manager.c")
        .file("apron/apron/ap_reducedproduct.c")
        .file("apron/apron/ap_lincons0.c")
        .file("apron/apron/ap_lincons1.c")
        .file("apron/apron/ap_dimension.c")
        .file("apron/apron/ap_var.c")
        // .file("apron/apron/test_texpr0.c")
        .file("apron/apron/ap_generator0.c")
        .file("apron/apron/ap_generator1.c")
        .file("apron/apron/ap_texpr0.c")
        .file("apron/apron/ap_texpr1.c")
        .file("apron/apron/ap_linearize.c")
        .file("apron/apron/ap_tcons0.c")
        .file("apron/apron/ap_tcons1.c")
        .file("apron/apron/ap_policy.c")
        .file("apron/apron/ap_generic.c")
        .file("apron/apron/ap_coeff.c")
        .file("apron/apron/ap_abstract0.c")
        .file("apron/apron/ap_abstract1.c")
        .file("apron/apron/ap_environment.c")
        .file("apron/apron/ap_disjunction.c")
        .file("src/apron_sys_wrapper.c")
        .compile("apron_mpq");

    // Apron linearize MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        .file("apron/apron/ap_linearize_aux.c")
        .compile("apron_linearize_mpq");

    // Apron linearize MPFR
    builder
        .clone()
        .flag("-DNUM_MPFR")
        .file("apron/apron/ap_linearize_aux.c")
        .compile("apron_linearize_mpfr");

    // Apron linearize DOUBLE
    builder
        .clone()
        .flag("-DNUM_DOUBLE")
        .file("apron/apron/ap_linearize_aux.c")
        .compile("apron_linearize_double");

    // Apron box MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        .file("apron/box/box_resize.c")
        .file("apron/box/box_internal.c")
        .file("apron/box/box_representation.c")
        .file("apron/box/box_otherops.c")
        .file("apron/box/box_assign.c")
        .file("apron/box/box_meetjoin.c")
        .file("apron/box/box_constructor.c")
        .file("apron/box/box_policy.c")
        .compile("apron_box");

    // Apron itv DOUBLE
    builder
        .clone()
        .flag("-DNUM_DOUBLE")
        .file("apron/itv/itv_linearize.c")
        .file("apron/itv/itv.c")
        .file("apron/itv/itv_linexpr.c")
        .compile("apron_itv_double");

    // Apron itv MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        .file("apron/itv/itv_linearize.c")
        .file("apron/itv/itv.c")
        .file("apron/itv/itv_linexpr.c")
        .compile("apron_itv_mpq");

    // Apron itv MPFR
    builder
        .clone()
        .flag("-DNUM_MPFR")
        .file("apron/itv/itv_linearize.c")
        .file("apron/itv/itv.c")
        .file("apron/itv/itv_linexpr.c")
        .compile("apron_itv_mpfr");

    // Apron octagons MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        .file("apron/octagons/oct_representation.c")
        .file("apron/octagons/oct_nary.c")
        .file("apron/octagons/oct_transfer.c")
        .file("apron/octagons/oct_print.c")
        .file("apron/octagons/oct_resize.c")
        .file("apron/octagons/oct_predicate.c")
        // // .file("apron/octagons/oct_test.c")
        .file("apron/octagons/oct_hmat.c")
        .file("apron/octagons/oct_closure.c")
        .compile("apron_oct_mpq");

    // Apron newpolka MPQ
    builder
        .clone()
        .flag("-DNUM_MPQ")
        .file("apron/newpolka/mf_qsort.c")
        .file("apron/newpolka/pk_closure.c")
        .file("apron/newpolka/pk_project.c")
        .file("apron/newpolka/pkeq.c")
        .file("apron/newpolka/pk_assign.c")
        .file("apron/newpolka/pk_internal.c")
        // .file("apron/newpolka/test_environment.c")
        .file("apron/newpolka/pk_user.c")
        .file("apron/newpolka/pk_extract.c")
        .file("apron/newpolka/pk_approximate.c")
        .file("apron/newpolka/pk_matrix.c")
        // .file("apron/newpolka/test0.c")
        .file("apron/newpolka/pk_constructor.c")
        .file("apron/newpolka/pk_representation.c")
        .file("apron/newpolka/pk_meetjoin.c")
        // .file("apron/newpolka/test1.c")
        .file("apron/newpolka/pk_test.c")
        .file("apron/newpolka/pk_resize.c")
        // .file("apron/newpolka/test.c")
        .file("apron/newpolka/pk_satmat.c")
        .file("apron/newpolka/pk_widening.c")
        .file("apron/newpolka/pk_vector.c")
        .file("apron/newpolka/pk_expandfold.c")
        .file("apron/newpolka/pk_bit.c")
        .file("apron/newpolka/pk_cherni.c")
        .compile("apron_pk_mpq");

    // Apron PPL
    cpp_builder
        .file("apron/ppl/ppl_user.cc")
        .file("apron/ppl/ppl_poly.cc")
        .file("apron/ppl/ppl_grid.cc")
        .compile("apron_ppl");

    // Link the PPL library
    println!("cargo:rustc-link-lib=ppl");

    // Apron Products
    builder
        .flag("-DNUM_MPQ")
        .file("apron/products/ap_pkgrid.c")
        .compile("apron_products");

    // For bindgen
    const INCLUDED_TYPES: &[&str] = &[
        "ap_dimperm_t",
        "ap_dimension_t",
        "ap_dimchange_t",
        "ap_dim_t",
        "ap_texpr0_t",
        "ap_abstract0_t",
        "ap_manager_t",
        "ap_lincons0_array_t",
        "ap_coeff_t",
        "ap_scalar_t",
        "ap_constyp_t",
        "ap_interval_t",
        "ap_tcons0_array_t",
        "ap_tcons0_t",
    ];
    const INCLUDED_FUNCTIONS: &[&str] = &[];
    const INCLUDED_VARS: &[&str] = &[];

    let mut builder = bindgen::Builder::default()
        .derive_default(true)
        .rustfmt_bindings(true)
        .header("src/apron_sys_wrapper.h")
        .clang_arg("-Iapron/num")
        .clang_arg("-Iapron/itv")
        .clang_arg("-Iapron/apron")
        .clang_arg("-Iapron/box")
        .clang_arg("-Iapron/ppl")
        .clang_arg("-Iapron/products")
        .clang_arg("-I/usr/include")
        .clang_arg("-std=c99")
        .clang_arg("-U__STRICT_ANSI__")
        .clang_arg("-fPIC")
        // .clang_arg("-O3")
        // .clang_arg("-g")
        // .clang_arg("-O0")
        .clang_arg("-DNDEBUG")
        .clang_arg("-DNUM_MPZ")
        .clang_arg("-Wcast-qual")
        .clang_arg("-Wswitch")
        .clang_arg("-Wall")
        .clang_arg("-Wextra")
        .clang_arg("-Wundef")
        .clang_arg("-Wcast-align")
        .clang_arg("-Wno-unused")
        .clang_arg("-Wno-unused-parameter")
        .clang_arg("-Wno-unused-function")
        .clang_arg("-Werror-implicit-function-declaration")
        .clang_arg("-Wbad-function-cast")
        .clang_arg("-Wstrict-prototypes");

    for t in INCLUDED_TYPES {
        builder = builder.whitelist_type(t);
    }
    for f in INCLUDED_FUNCTIONS {
        builder = builder.whitelist_function(f);
    }
    for v in INCLUDED_VARS {
        builder = builder.whitelist_var(v);
    }
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/apron_sys_wrapper.h");
    println!("cargo:rerun-if-changed=src/apron_sys_wrapper.c");
}

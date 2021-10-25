#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

/// Bindings for the Apron numerical abstract domain library
use gmp_mpfr_sys::gmp::mpq_t;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern "C" {
    // ap_manager_t* box_manager_alloc ()
    pub fn box_manager_alloc() -> *mut ap_manager_t;

    // void ap_manager_free (ap_manager_t* man)
    pub fn ap_manager_free(man: *mut ap_manager_t);

    // ap_abstract0_t* ap_abstract0_top (ap_manager_t* man, size_t intdim, size_t realdim)
    pub fn ap_abstract0_top(
        man: *mut ap_manager_t,
        intdim: libc::size_t,
        realdim: libc::size_t,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_bottom (ap_manager_t* man, size_t intdim, size_t realdim)
    pub fn ap_abstract0_bottom(
        man: *mut ap_manager_t,
        intdim: libc::size_t,
        realdim: libc::size_t,
    ) -> *mut ap_abstract0_t;

    // void ap_abstract0_free (ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_free(man: *mut ap_manager_t, a: *mut ap_abstract0_t);

    // bool ap_abstract0_is_bottom (ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_is_bottom(man: *mut ap_manager_t, a: *mut ap_abstract0_t) -> bool;

    // bool ap_abstract0_is_top (ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_is_top(man: *mut ap_manager_t, a: *mut ap_abstract0_t) -> bool;

    // ap_abstract0_t* ap_abstract0_assign_texpr (ap_manager_t* man, bool destructive, ap_abstract0_t* org, ap_dim_t dim, ap_texpr0_t* expr, ap_abstract0_t* dest)
    pub fn ap_abstract0_assign_texpr(
        man: *mut ap_manager_t,
        destructive: bool,
        org: *mut ap_abstract0_t,
        dim: ap_dim_t,
        expr: *mut ap_texpr0_t,
        dest: *mut ap_abstract0_t,
    ) -> *mut ap_abstract0_t;

    // ap_texpr0_t* ap_texpr0_binop(ap_texpr_op_t op, ap_texpr0_t* opA, ap_texpr0_t* opB, ap_texpr_rtype_t type, ap_texpr_rdir_t dir)
    pub fn ap_texpr0_binop(
        op: ap_texpr_op_t,
        opA: *mut ap_texpr0_t,
        opB: *mut ap_texpr0_t,
        _type: ap_texpr_rtype_t,
        dir: ap_texpr_rdir_t,
    ) -> *mut ap_texpr0_t;

    // ap_texpr0_t* ap_texpr0_unop(ap_texpr_op_t op, ap_texpr0_t* opA, ap_texpr_rtype_t type, ap_texpr_rdir_t dir)
    pub fn ap_texpr0_unop(
        op: ap_texpr_op_t,
        opA: *mut ap_texpr0_t,
        _type: ap_texpr_rtype_t,
        dir: ap_texpr_rdir_t,
    ) -> *mut ap_texpr0_t;

    // bool ap_abstract0_is_eq (ap_manager_t* man, ap_abstract0_t* a1, ap_abstract0_t* a2)
    pub fn ap_abstract0_is_eq(
        man: *mut ap_manager_t,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> bool;

    // bool ap_abstract0_is_leq (ap_manager_t* man, ap_abstract0_t* a1, ap_abstract0_t* a2)
    pub fn ap_abstract0_is_leq(
        man: *mut ap_manager_t,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> bool;

    // ap_texpr0_t* ap_texpr0_cst_scalar_mpq(mpq_t mpq)
    pub fn ap_texpr0_cst_scalar_mpq(mpq: *const mpq_t) -> *mut ap_texpr0_t;

    // ap_texpr0_t* ap_texpr0_cst_scalar_int(long int num);
    pub fn ap_texpr0_cst_scalar_int(num: libc::c_long) -> *mut ap_texpr0_t;

    // ap_texpr0_t* ap_texpr0_dim(ap_dim_t dim)
    pub fn ap_texpr0_dim(dim: ap_dim_t) -> *mut ap_texpr0_t;

    // ap_dimchange_t* ap_dimchange_alloc(size_t intdim, size_t realdim)
    pub fn ap_dimchange_alloc(intdim: libc::size_t, realdim: libc::size_t) -> *mut ap_dimchange_t;

    // ap_abstract0_t* ap_abstract0_add_dimensions(ap_manager_t* man, bool destructive, ap_abstract0_t* a, ap_dimchange_t* dimchange, bool project)
    pub fn ap_abstract0_add_dimensions(
        man: *mut ap_manager_t,
        destructive: bool,
        a: *mut ap_abstract0_t,
        dimchange: *mut ap_dimchange_t,
        project: bool,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_remove_dimensions(ap_manager_t* man, bool destructive, ap_abstract0_t* a, ap_dimchange_t* dimchange)
    pub fn ap_abstract0_remove_dimensions(
        man: *mut ap_manager_t,
        destructive: bool,
        a: *mut ap_abstract0_t,
        dimchange: *mut ap_dimchange_t,
    ) -> *mut ap_abstract0_t;

    // ap_dimension_t ap_abstract0_dimension(ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_dimension(man: *mut ap_manager_t, a: *mut ap_abstract0_t)
        -> ap_dimension_t;

    // static inline void ap_dimchange_free(ap_dimchange_t* dimchange)
    pub fn ap_dimchange_free_wrapper(dimchange: *mut ap_dimchange_t);

    // ap_texpr0_t* ap_texpr0_cst_interval_mpq(mpq_t inf, mpq_t sup)
    pub fn ap_texpr0_cst_interval_mpq(inf: mpq_t, sup: mpq_t) -> *mut ap_texpr0_t;

    // void ap_texpr0_free(ap_texpr0_t* expr)
    pub fn ap_texpr0_free(expr: *mut ap_texpr0_t);

    // ap_lincons0_array_t ap_abstract0_to_lincons_array(ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_to_lincons_array(
        man: *mut ap_manager_t,
        a: *mut ap_abstract0_t,
    ) -> ap_lincons0_array_t;

    // void ap_lincons0_array_clear(ap_lincons0_array_t* array)
    pub fn ap_lincons0_array_clear(array: *mut ap_lincons0_array_t);

    // bool ap_coeff_zero(ap_coeff_t* coeff)
    pub fn ap_coeff_zero(coeff: *mut ap_coeff_t) -> bool;

    // size_t ap_abstract0_size (ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_size(man: *mut ap_manager_t, a: *mut ap_abstract0_t) -> libc::size_t;

    // ap_dimperm_t* ap_dimperm_alloc(size_t size)
    pub fn ap_dimperm_alloc(size: libc::size_t) -> *mut ap_dimperm_t;

    // ap_abstract0_t* ap_abstract0_permute_dimensions(ap_manager_t* man, bool destructive, ap_abstract0_t* a, ap_dimperm_t* perm)
    pub fn ap_abstract0_permute_dimensions(
        man: *mut ap_manager_t,
        destructive: bool,
        a: *mut ap_abstract0_t,
        perm: *mut ap_dimperm_t,
    ) -> *mut ap_abstract0_t;

    // void ap_dimperm_free(ap_dimperm_t* dimperm) {
    pub fn ap_dimperm_free_wrapper(dimperm: *mut ap_dimperm_t);

    // ap_abstract0_t* ap_abstract0_join(ap_manager_t* man, bool destructive, ap_abstract0_t* a1, ap_abstract0_t* a2)
    pub fn ap_abstract0_join(
        man: *mut ap_manager_t,
        destructive: bool,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_meet(ap_manager_t* man, bool destructive, ap_abstract0_t* a1, ap_abstract0_t* a2)
    pub fn ap_abstract0_meet(
        man: *mut ap_manager_t,
        destructive: bool,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_widening(ap_manager_t* man, ap_abstract0_t* a1, ap_abstract0_t* a2)
    pub fn ap_abstract0_widening(
        man: *mut ap_manager_t,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_copy(ap_manager_t* man, ap_abstract0_t* a)
    pub fn ap_abstract0_copy(man: *mut ap_manager_t, a: *mut ap_abstract0_t)
        -> *mut ap_abstract0_t;

    // ap_interval_t* ap_abstract0_bound_dimension(ap_manager_t* man, ap_abstract0_t* a, ap_dim_t dim)
    pub fn ap_abstract0_bound_dimension(
        man: *mut ap_manager_t,
        a: *mut ap_abstract0_t,
        dim: ap_dim_t,
    ) -> *mut ap_interval_t;

    // bool ap_interval_is_top(ap_interval_t* interval)
    pub fn ap_interval_is_top(interval: *mut ap_interval_t) -> bool;

    // void ap_interval_free(ap_interval_t* itv)
    pub fn ap_interval_free(itv: *mut ap_interval_t);

    // int ap_scalar_infty(ap_scalar_t* scalar)
    pub fn ap_scalar_infty(scalar: *mut ap_scalar_t) -> libc::c_int;

    // ap_manager_t* oct_manager_alloc(void)
    pub fn oct_manager_alloc() -> *mut ap_manager_t;

    // ap_manager_t* pk_manager_alloc(bool strict)
    pub fn pk_manager_alloc(strict: bool) -> *mut ap_manager_t;

    // ap_lincons0_array_t ap_lincons0_array_make(size_t size)
    pub fn ap_lincons0_array_make(size: libc::size_t) -> ap_lincons0_array_t;

    // ap_abstract0_t* ap_abstract0_widening_threshold(ap_manager_t* man, ap_abstract0_t* a1, ap_abstract0_t* a2, ap_lincons0_array_t* array)
    pub fn ap_abstract0_widening_threshold(
        man: *mut ap_manager_t,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
        array: *mut ap_lincons0_array_t,
    ) -> *mut ap_abstract0_t;

    // ap_abstract0_t* ap_abstract0_forget_array(ap_manager_t* man, bool destructive, ap_abstract0_t* a, ap_dim_t* tdim, size_t size, bool project)
    pub fn ap_abstract0_forget_array(
        man: *mut ap_manager_t,
        destructive: bool,
        a: *mut ap_abstract0_t,
        tdim: *mut ap_dim_t,
        size: libc::size_t,
        project: bool,
    ) -> *mut ap_abstract0_t;

    // ap_tcons0_array_t ap_tcons0_array_make(size_t size)
    pub fn ap_tcons0_array_make(size: libc::size_t) -> ap_tcons0_array_t;

    // ap_tcons0_t ap_tcons0_make(ap_constyp_t constyp, ap_texpr0_t* texpr, ap_scalar_t* scalar) {
    pub fn ap_tcons0_make_wrapper(
        constyp: ap_constyp_t,
        texpr: *mut ap_texpr0_t,
        scalar: *mut ap_scalar_t,
    ) -> ap_tcons0_t;

    // ap_abstract0_t* ap_abstract0_meet_tcons_array(ap_manager_t* man, bool destructive, ap_abstract0_t* a, ap_tcons0_array_t* array)
    pub fn ap_abstract0_meet_tcons_array(
        man: *mut ap_manager_t,
        destructive: bool,
        a: *mut ap_abstract0_t,
        array: *mut ap_tcons0_array_t,
    ) -> *mut ap_abstract0_t;

    // void ap_tcons0_array_clear(ap_tcons0_array_t* array)
    pub fn ap_tcons0_array_clear(array: *mut ap_tcons0_array_t);

    // ap_manager_t* pkeq_manager_alloc(void);
    pub fn pkeq_manager_alloc() -> *mut ap_manager_t;

    // ap_manager_t* ap_ppl_poly_manager_alloc(bool strict);
    pub fn ap_ppl_poly_manager_alloc(strict: bool) -> *mut ap_manager_t;

    // ap_manager_t* ap_ppl_grid_manager_alloc(void);
    pub fn ap_ppl_grid_manager_alloc() -> *mut ap_manager_t;

    // ap_manager_t* ap_pkgrid_manager_alloc(ap_manager_t* manpk, ap_manager_t* manpplgrid);
    pub fn ap_pkgrid_manager_alloc(
        manpk: *mut ap_manager_t,
        manpplgrid: *mut ap_manager_t,
    ) -> *mut ap_manager_t;

    // bool ap_linexpr0_is_integer (ap_linexpr0_t* e, size_t intdim)
    pub fn ap_linexpr0_is_integer(e: *mut ap_linexpr0_t, intdim: libc::size_t) -> bool;

    // ap_abstract0_t* ap_abstract0_oct_narrowing(ap_manager_t* man, ap_abstract0_t* a1, ap_abstract0_t* a2);
    pub fn ap_abstract0_oct_narrowing(
        man: *mut ap_manager_t,
        a1: *mut ap_abstract0_t,
        a2: *mut ap_abstract0_t,
    ) -> *mut ap_abstract0_t;
}

#include <ap_dimension.h>
#include <ap_tcons0.h>

void ap_dimchange_free_wrapper(ap_dimchange_t* dimchange) {
  ap_dimchange_free(dimchange);
}

void ap_dimperm_free_wrapper(ap_dimperm_t* dimperm) {
  ap_dimperm_free(dimperm);
}
ap_tcons0_t ap_tcons0_make_wrapper(ap_constyp_t constyp, ap_texpr0_t* texpr, ap_scalar_t* scalar) {
  return ap_tcons0_make(constyp, texpr, scalar);
}

fn main() {
    /* Common case: The double precision result is fine. */
    if (ui & 0x1fffffff) != 0x10000000  /* not a halfway case */
        || e == 0x7ff /* NaN */
    || (result - xy == z as f64 && result - z as f64 == xy) /* exact */
        || fegetround() != FE_TONEAREST
    /* not round-to-nearest */
    {}
}

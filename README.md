`satir` is an implementation of a CDCL SAT solver in Rust.  Note that it is a work in progress and does not yet actually work.

The implementation follows [minisat](http://minisat.se/).  So far, the most interesting part of the implementation is its use of the [slice_dst](https://docs.rs/slice-dst/latest/slice_dst/) package to embed clause literals in clauses, removing a typical source of indirections.

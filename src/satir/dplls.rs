use crate::satir::core;
use crate::satir::constraint;
use crate::satir::clause;

struct Env {
    decision_stack :: Vec<core::Literal>
}

// Clauses
use cdcl::core;

pub struct Clause {
    activity: f64,
    lit_count: i16,
    flags: i16,
    literals: [Literal],
}

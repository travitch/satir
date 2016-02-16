use tagged;
use cdcl::core;

pub struct DecisionIndex(usize);

pub struct SolverEnv {
    decision_stack: tagged::TaggedVec<DecisionIndex,core::Literal>,
    decision_boundaries: Vec<DecisionIndex>,
    assignment: tagged::TaggedVec<core::Variable, core::Value>,
    variable_levels: tagged::TaggedVec<core::Variable, i32>,
    variable_activity: tagged::TaggedVec<core::Variable, f64>,
//    decision_reasons: TaggedArray<'a,core::Variable, Option<'a &Clause>>,
}

use tagged;
use cdcl::core;
use cdcl::constraint;

#[derive(Clone,Copy)]
pub struct DecisionIndex(usize);

pub struct SolverRoot {
    problem_constraints: Vec<Box<constraint::Constraint>>,
    learned_constraints: Vec<Box<constraint::Constraint>>,
}

pub struct SolverEnv<'a> {
    root: &'a mut SolverRoot,
    decision_stack: tagged::TaggedVec<DecisionIndex,core::Literal>,
    decision_boundaries: Vec<DecisionIndex>,
    assignment: tagged::TaggedVec<core::Variable, core::Value>,
    propagation_queue: DecisionIndex,
    constraint_id_src: u64,
    pub constraint_increment: f64,
    pub decision_reasons: tagged::TaggedVec<core::Variable, Option<&'a constraint::Constraint>>,
    variable_activity: tagged::TaggedVec<core::Variable, f64>,
    variable_levels: tagged::TaggedVec<core::Variable, i32>,
    variable_increment: f64,
    pub watchlist: tagged::TaggedVec<core::Literal, Vec<&'a constraint::Constraint>>,
}

pub const ACTIVITY_CAP : f64 = 1e100;

pub fn rescale_activity(env: &mut SolverEnv) -> () {
    unimplemented!();
}

pub fn increment_decision_level(env: &mut SolverEnv) -> () {
    let dl = env.decision_stack.len();
    env.decision_boundaries.push(DecisionIndex(dl));
}

pub fn decision_level(env: &SolverEnv) -> i32 {
    env.decision_boundaries.len() as i32
}

pub fn assign_variable_value<'a>(env: &mut SolverEnv<'a>, var: core::Variable, val: core::Value, reason: Option<&'a constraint::Constraint>) -> () {
    let dl = decision_level(env);
    env.decision_reasons[var] = reason;
    env.assignment[var] = val;
    env.variable_levels[var] = dl;
}

pub fn reset_variable(env: &mut SolverEnv, var: core::Variable) -> () {
    env.assignment[var] = core::UNASSIGNED;
    env.variable_levels[var] = -1;
    env.decision_reasons[var] = None;
}

pub fn assert_literal<'a>(env: &mut SolverEnv<'a>, lit: core::Literal, reason: Option<&'a constraint::Constraint>) -> () {
    let var = core::variable(lit);
    let val = core::satisfy_literal(lit);
    assign_variable_value(env, var, val, reason);
    env.decision_stack.push(lit);
}

pub fn try_assert_literal<'a>(env: &mut SolverEnv<'a>, lit: core::Literal, reason: Option<&'a constraint::Constraint>) -> bool {
    let var = core::variable(lit);
    let val = core::satisfy_literal(lit);
    let cur_val = env.assignment[var];
    if cur_val != core::UNASSIGNED {
        return cur_val == val;
    } else {
        assign_variable_value(env, var, val, reason);
        env.decision_stack.push(lit);
        return true;
    }
}

pub fn literal_value(env : &SolverEnv, lit : core::Literal) -> core::Value {
    let var_val = env.assignment[core::variable(lit)];
    core::lit_val(lit, var_val)
}

/* Note [SplitStruct]

The SolverEnv struct is split in two because rust does not support
saying "this reference only lives as long as the contents of the
containing stuct".  The env needs to have references to constraints,
which are owned by the `problem_constraints` and `learned_constraints`
fields of the struct.  The references require a lifetime, so we need
to split the struct in order to have a lifetime to talk about.

Also note that the Constraint storage refers to Constraints through a
Box; this is required because Constraint is an interface, and
different constraint types can have different storage requirements.
This means that they must be heap allocated (i.e., boxed).

*/

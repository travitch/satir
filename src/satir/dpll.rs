use crate::satir::core;
use crate::satir::constraint;
use crate::satir::clause;
use crate::satir::tagged;

/// Solver statistics tracked for reporting purposes
struct Statistics {
    /// The number of conflicts encountered while solving
    conflicts : usize,
    /// The total number of decisions attempted
    decisions : usize,
    /// The total number of times that the unit propagation rule has been applied
    propagations : usize
}

fn empty_statistics() -> Statistics {
    Statistics {
        conflicts : 0,
        decisions : 0,
        propagations : 0
    }
}

struct Env {
    /// The original clauses of the problem
    problem : Vec<clause::Clause>,
    /// The decisions that have been made (in order)
    decision_stack : Vec<core::Literal>,
    /// The current assignment (which could be derived from the decision stack)
    assignment : tagged::TaggedVec<core::Variable, core::Value>,
    /// Statistics from one run of the algorithm
    statistics : Statistics
}

struct PreprocessResult {
    /// Variables with implied initial assignments
    initial_assignment : tagged::TaggedVec<core::Variable, core::Value>,
    /// Variables for which we have detected a conflict during preprocessing
    conflict_vars : Vec<core::Variable>
}

/// Preprocess the formula to both simplify it and identify any initial conflicts
///
/// 1. Remove empty clauses
///
/// 2. Remove singleton clauses and record them in the `PreprocessResult` as
///    part of an initial assignment (to be folded into the env)
fn preprocess(clauses : &mut Vec<clause::Clause>, next_var : &core::Variable) -> PreprocessResult {
    let mut pr = PreprocessResult {
        initial_assignment : tagged::TaggedVec::new(),
        conflict_vars : Vec::new()
    };

    pr.initial_assignment.ensure_index(next_var, core::UNASSIGNED);

    clauses.retain(|cl| {
        if cl.lit_count() == 0 {
            return false;
        } else if cl.lit_count() == 1 {
            let single_lit = cl[0];
            let current_assign = pr.initial_assignment[core::variable(single_lit)];
            if current_assign == core::UNASSIGNED {
                // We can assign this variable and discard the clause
                return false;
            } else if core::satisfy_literal(single_lit) == current_assign {
                // We can eliminate the clause because we already have this assignment
                return false;
            } else {
                // We have found a conflict... no need to remove it, though we could
                pr.conflict_vars.push(core::variable(single_lit));
                return true;
            }
        } else {
            return true;
        }
    });

    pr
}

enum PropagateResult {
    Conflict,
    NoConflict
}

fn propagate_units(env : &mut Env, lit : core::Literal) -> PropagateResult {
    unimplemented!()
}

/// Pick the next literal to set
///
/// This can be either an arbitrary choice or taken from a list of implied
/// decisions (e.g., due to watched literals)
fn decide(env : &mut Env) -> Option<core::Literal> {
    unimplemented!()
}

pub fn solve(mut clauses : Vec<clause::Clause>, next_var : core::Variable) -> core::Result {
    // If there is an obvious syntactic conflict, return early
    //
    // Those can arise if there are conflicting unit clauses, so propagate units
    let pp_result = preprocess(&mut clauses, &next_var);
    if pp_result.conflict_vars.len() > 0 {
        return core::Result::Unsat;
    }

    let mut env = Env {
        problem : clauses,
        decision_stack : Vec::new(),
        assignment : pp_result.initial_assignment,
        statistics : empty_statistics()
    };

    // Next, decide and propagate units until we have completed the assignment
    // or exhausted our possible assignments
    while let Some(next_lit) = decide(&mut env) {
        propagate_units(&mut env, next_lit);
    }


    unimplemented!()
}

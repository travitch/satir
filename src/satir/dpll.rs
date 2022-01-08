use std::collections::{BTreeSet, VecDeque};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::satir::core::{Literal, Variable, Value};
use crate::satir::core;
use crate::satir::clause::{Clause, ClauseId};
use crate::satir::tagged::TaggedVec;

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
    problem : TaggedVec<ClauseId, Clause>,
    /// The decisions that have been made (in order)
    decision_stack : Vec<Literal>,
    /// The current assignment (which could be derived from the decision stack)
    assignment : TaggedVec<Variable, Value>,
    /// Maintain an index of variables to the clauses watching them; note that
    /// we have to refer to clauses by their index into the clause database
    ///
    /// NOTE: Because these are unadorned indexes, this will be a bit trickier
    /// once we learn (and delete) clauses.
    watchlist : TaggedVec<Literal, BTreeSet<ClauseId>>,
    /// The order to decide variables; note that this *can* be updated
    /// dynamically. Also note that the variables in this could potentially
    /// already be decided due to e.g., the watched literals queue
    variable_order : PriorityQueue<Variable, OrderedFloat<f32>>,
    /// Literals that we must assert next due to findings (via two-watched
    /// literals) during unit propagation; these take priority over the natural
    /// variable ordering
    decision_queue : VecDeque<Literal>,
    /// Statistics from one run of the algorithm
    statistics : Statistics
}

impl Env {
    // Evaluate this literal with respect to the current assignment
    fn value_of(&self, lit : Literal) -> Value {
        lit.under_value(self.assignment[lit.variable()])
    }
}

struct PreprocessResult {
    /// Variables with implied initial assignments
    initial_assignment : TaggedVec<Variable, Value>,
    /// Variables for which we have detected a conflict during preprocessing
    conflict_vars : Vec<Variable>
}

/// Preprocess the formula to both simplify it and identify any initial conflicts
///
/// 1. Remove empty clauses
///
/// 2. Remove singleton clauses and record them in the `PreprocessResult` as
///    part of an initial assignment (to be folded into the env)
///
/// After this, the clause database contains clauses with at least two literals
///
/// FIXME: Look for intra-clause inconsistencies (i.e., x /\ !x)
fn preprocess(clauses : &mut Vec<Clause>, next_var : &Variable) -> PreprocessResult {
    let mut pr = PreprocessResult {
        initial_assignment : TaggedVec::new(),
        conflict_vars : Vec::new()
    };

    pr.initial_assignment.ensure_index(next_var, Value::UNASSIGNED);

    clauses.retain(|cl| {
        if cl.lit_count() == 0 {
            return false;
        } else if cl.lit_count() == 1 {
            let single_lit = cl[0];
            let current_assign = pr.initial_assignment[single_lit.variable()];
            if current_assign == Value::UNASSIGNED {
                // We can assign this variable and discard the clause
                return false;
            } else if single_lit.satisfy() == current_assign {
                // We can eliminate the clause because we already have this assignment
                return false;
            } else {
                // We have found a conflict... no need to remove it, though we could

                pr.conflict_vars.push(single_lit.variable());
                return true;
            }
        } else {
            return true;
        }
    });

    pr
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum PropagateResult {
    Conflict,
    NoConflict
}

// Enqueue a literal, with some consistency checks to ensure that we don't
// enqueue multiple times
//
// Note that this can detect a conflict if an earlier propagation at this
// decision level enqueued a conflict.
fn enqueue(env : &mut Env, lit : Literal) -> PropagateResult {
    let val = env.value_of(lit);
    if val == Value::UNASSIGNED {
        // Assign immediately; note that we still enqueue because we have to
        // propagate units still
        env.decision_stack.push(lit);
        env.assignment[lit.variable()] = lit.satisfy();
        env.decision_queue.push_back(lit);
        return PropagateResult::NoConflict;
    } else {
        if val == Value::LIFTED_FALSE {
            return PropagateResult::Conflict;
        } else {
            // Already assigned, no need to re-enqueue
            return PropagateResult::NoConflict;
        }
    }
}

/// Given the new decision `lit`, propagate units
///
/// We only care about clauses watching `¬lit` (as clauses watching `lit` are
/// now satisfied).  For each clause watching `¬lit`:
///
/// 1. Set new watches,
///
/// 2. Identify units (propagate), or
///
/// 3. Recognize conflicts and initiate backtracking
fn propagate_units(env : &mut Env, lit : Literal) -> PropagateResult {
    // We are here because we decided `lit`, which means either l0 or l1 were
    // watching ¬lit
    let false_lit = lit.negate();
    // We need to iterate over the watchers; however, we also need to be able to
    // mutate the watch index as we find new watches, which requires
    // mutability. To do that, we take local ownership of this set of watchers
    // and establish a separate list of watchers of this literal.
    //
    // If we hit a conflict, we need to copy over the remaining former watchers
    let watchers = std::mem::replace(&mut env.watchlist[false_lit], BTreeSet::new());

    // For each one of these, find a new watch
    //
    // After this, either we have found a new watch for every clause or there is
    // a conflict and we will backtrack (undoing this decision, so that we don't
    // need to undo any watches
    let mut idx_iter = watchers.iter();
    while let Some(idx) = idx_iter.next() {
        env.statistics.propagations += 1;
        let cl = &mut env.problem[*idx];
        if cl[1] != false_lit {
            cl[0] = cl[1];
            cl[1] = false_lit;
        }

        // let other_lit = cl[0];
        if cl[0].under_value(env.assignment[cl[0].variable()]) == Value::LIFTED_TRUE {
            // The clause is satisfied - no need to update its watches (but we
            // need to put the second watch back on cl[1])
            env.watchlist[false_lit].insert(cl.identifier());
            continue;
        }

        // Find a new literal to watch for cl[1], starting from cl[2]
        //
        // We can watch anything that is not FALSE (i.e., TRUE and UNASSIGNED are acceptable)
        let mut found_watch = false;
        for lit_num in 2.. cl.lit_count() {
            if cl[lit_num].under_value(env.assignment[cl[lit_num].variable()]) == Value::LIFTED_FALSE {
                continue;
            }

            let tmp_lit = cl[1];
            cl[1] = cl[lit_num];
            cl[lit_num] = tmp_lit;
            // Add the new watch to the index (we don't need to remove this one
            // from the index, as we started by obliterating the index)
            //
            // Note that we have to negate the literal, as we want to find a new
            // watch when cl[1] is *not* satisfied.
            env.watchlist[false_lit].insert(cl.identifier());
            found_watch = true;
            break;
        }

        // If there is nothing to watch *and* the other watch is UNASSIGNED, we
        // must satisfy this clause by asserting that literal. We would add it
        // to the queue.
        //
        // If the other literal is TRUE, that is okay; we can leave the watch
        // dead (this clause is satisfied)
        //
        // If there is nothing to watch and the other watched literal is FALSE,
        // we have a conflict
        if found_watch {
            continue;
        }
        if cl[0].under_value(env.assignment[cl[0].variable()]) == Value::LIFTED_FALSE {
            // We did not find a watch and the other literal is false, so we
            // have a conflict. Preserve the rest of the watches that we didn't
            // update.
            env.watchlist[false_lit].insert(cl.identifier());
            while let Some(clause_id) = idx_iter.next() {
                env.watchlist[false_lit].insert(*clause_id);
            }

            env.statistics.conflicts += 1;
            env.decision_queue.clear();
            return PropagateResult::Conflict;
        } else {
            // Otherwise, we found a unit clause and can just queue up the
            // literal required to satisfy it (and need to continue propagating
            // units)
            let enq_lit = cl[0];
            let clause_id = cl.identifier();
            if enqueue(env, enq_lit) == PropagateResult::Conflict {
                env.watchlist[false_lit].insert(clause_id);
                while let Some(clause_id) = idx_iter.next() {
                    env.watchlist[false_lit].insert(*clause_id);
                }

                env.statistics.conflicts += 1;
                env.decision_queue.clear();
                return PropagateResult::Conflict;
            }
        }
    }

    PropagateResult::NoConflict
}

/// Pick the next literal to set
///
/// This can be either an arbitrary choice or taken from a list of implied
/// decisions (e.g., due to watched literals)
fn next_decision(env : &mut Env) -> Option<Literal> {
    // See if anything in the immediate queue is unassigned; it could be the
    // case that something in here was actually assigned already
    while let Some(l) = env.decision_queue.pop_front() {
        if env.assignment[l.variable()] == Value::UNASSIGNED {
            return Some(l);
        }
    }

    // FIXME: Find some way to persist the priority so that we could restore it
    // if we re-add the variable to the decision queue
    //
    // Note: we can just use variable activity for this
    loop {
        match env.variable_order.pop() {
            Some((v, _)) => {
                if env.assignment[v] == Value::UNASSIGNED {
                    return Some(v.to_positive_literal());
                }
            }
            None => {
                return None;
            }
        }
    }
}

/// Look at the last decision we made and undo it
///
/// This involves removing the assignment and undoing any relevant modifications
/// made during unit propagation
fn undo_last_decision(env : &mut Env) -> () {
    match env.decision_stack.pop() {
        None => {}
        Some(l) => {
            env.assignment[l.variable()] = Value::UNASSIGNED;
            // FIXME: Choose a new priority (likely based on variable activity)
            env.variable_order.push(l.variable(), OrderedFloat(0.0));
        }
    }
}

/// Assign a trivial and not particularly useful priority to each variable
///
/// The priority is based just on the order variables are encountered
fn initial_variable_order(clauses : &Vec<Clause>) -> PriorityQueue<Variable, OrderedFloat<f32>> {
    let mut priority = 0;
    let mut q = PriorityQueue::new();
    let mut seen = BTreeSet::new();
    for c in clauses {
        for idx in 0..c.lit_count() {
            let v = c[idx].variable();
            if seen.contains(&v) {
                continue;
            }

            q.push(v, OrderedFloat(priority as f32));
            priority += 1;
            seen.insert(v);
        }
    }

    q
}

/// Fill in the watchlist index; this must come after preprocessing, as we
/// require that all clauses have at least two literals
///
/// FIXME: Split preprocessing into its own module and capture that invariant in
/// a newtype
///
/// The convention is that the first two literals of each clause are watched, so
/// build the reverse index based on the current literal ordering.
fn initialize_watchlist(clauses : &TaggedVec<ClauseId, Clause>,
                        watch_index : &mut TaggedVec<Literal, BTreeSet<ClauseId>>)
{
    let mut clause_iter = clauses.iter();
    while let Some(cl) = clause_iter.next() {
        watch_index.ensure_index(&cl[0], BTreeSet::new());
        watch_index.ensure_index(&cl[1], BTreeSet::new());
        let cid = cl.identifier();
        watch_index[cl[0]].insert(cid);
        watch_index[cl[1]].insert(cid);
    }
}

/// Put our clauses into a `TaggedVec` with safer types
///
/// This allocates all of the `ClauseId`s, ensuring that each clause's
/// `ClauseId` matches its index in the `TaggedVec`.
fn intern_clauses(clauses : Vec<Clause>) -> TaggedVec<ClauseId, Clause> {
    // Ensure that the index of each clause matches its ClauseId (so that we can
    // maintain the watchlist index)
    let mut numbered_clauses = TaggedVec::new();
    let mut clause_num = 0;
    let mut clause_iter = clauses.into_iter();
    while let Some(mut cl) = clause_iter.next() {
        let clause_id = ClauseId(clause_num);
        clause_num += 1;
        *cl.identifier_mut() = clause_id;
        numbered_clauses.push(cl);
    }

    numbered_clauses
}

pub fn solve(mut clauses : Vec<Clause>, next_var : Variable) -> core::Result {
    // If there is an obvious syntactic conflict, return early
    //
    // Those can arise if there are conflicting unit clauses, so propagate units
    let pp_result = preprocess(&mut clauses, &next_var);
    if pp_result.conflict_vars.len() > 0 {
        return core::Result::Unsat;
    }
    // This is computed early just because we can't borrow it multiple times
    // while constructing `Env`
    let init_var_order = initial_variable_order(&clauses);
    let numbered_clauses = intern_clauses(clauses);

    // NOTE: This must come after preprocessing since we require all clauses to
    // have at least two literals
    let mut watch_index = TaggedVec::new();
    initialize_watchlist(&numbered_clauses, &mut watch_index);

    let mut env = Env {
        problem : numbered_clauses,
        decision_stack : Vec::new(),
        assignment : pp_result.initial_assignment,
        watchlist : TaggedVec::new(),
        variable_order : init_var_order,
        decision_queue : VecDeque::new(),
        statistics : empty_statistics()
    };


    // Next, decide and propagate units until we have completed the assignment
    // or exhausted our possible assignments
    while let Some(next_lit) = next_decision(&mut env) {
        match propagate_units(&mut env, next_lit) {
            PropagateResult::NoConflict => {
                // No special action - decide an assignment for the next
                // variable
            }
            PropagateResult::Conflict => {
                undo_last_decision(&mut env);
                let next_lit = next_lit.negate();
                env.decision_queue.push_back(next_lit);
            }
        }
    }


    return core::Result::Sat;
}

use cdcl::core;
use cdcl::constraint;
use cdcl::env;
use cdcl::watchlist;

pub struct Clause {
    id: u64,
    activity: f64,
    lit_count: u16,
    flags: u16,
    literals: [core::Literal],
}

impl constraint::Constraint for Clause {
    fn remove<'a>(&self, con : &'a constraint::Constraint, env: &mut env::SolverEnv<'a>) -> () {
        if self.lit_count >= 1 {
            watchlist::unwatch_literal(env, con, self.literals[0]);

            if self.lit_count >= 2 {
                watchlist::unwatch_literal(env, con, self.literals[1]);
            }
        }
    }

    fn propagate<'a>(&mut self, con : &'a constraint::Constraint, env: &mut env::SolverEnv<'a>, false_lit : core::Literal) -> constraint::PropagationResult {
        let other_lit = normalize_watched_literals(self, false_lit);
        let other_val = env::literal_value(env, other_lit);

        if other_val == core::LIFTED_TRUE {
            return constraint::PropagationResult::KeepWatch;
        }

        let n_lits = self.lit_count as usize;
        for ix in (2..n_lits - 1).rev() {
            let lit = self.literals[ix];
            let lit_val = env::literal_value(env, lit);
            if lit_val == core::LIFTED_FALSE {
                continue;
            }

            swap_literals(self, 1, ix);
            watchlist::watch_literal(env, con, lit);
            return constraint::PropagationResult::NewWatch;
        }

        // There was no new literal to watch, so we have a unit
        // clause.  We can try to assert the resulting literal, which
        // could actually fail.
        if env::try_assert_literal(env, other_lit, Some(con)) {
            return constraint::PropagationResult::KeepWatch;
        } else {
            return constraint::PropagationResult::Conflict;
        }
    }

    /*

    Remove the false literals from the clause.  If the clause is
    satisfied, return true (so that it will be removed).

    */
    fn simplify<'a>(&mut self, con : &'a constraint::Constraint, env: &mut env::SolverEnv<'a>) -> bool {
        for ix in (0..self.lit_count as usize).rev() {
            let l = self.literals[ix];
            let val = env::literal_value(env, l);
            if val == core::LIFTED_FALSE {
                // l is known to be false, so we can remove it from
                // the clause.
                let o_new_lit = remove_literal(self, ix);
                if ix < 2 {
                    match o_new_lit {
                        None => (),
                        Some(new_lit) => watchlist::watch_literal(env, con, new_lit),
                    }
                }
            } else if val == core::LIFTED_TRUE {
                return true;
            }
        }

        // If we only have a single literal remaining, we must assert it,
        // which means that we must also have it removed
        if self.lit_count == 1 {
            env::assert_literal(env, self.literals[0], None);
            return true;
        } else {
            return false;
        }
    }

    fn reason(&mut self, env: &mut env::SolverEnv, conflict_lit : Option<core::Literal>) -> &[core::Literal] {
        bump_clause_activity(env, self);
        let start_index : usize = match conflict_lit {
            None => 0,
            Some(_) => 1,
        };
        &self.literals[start_index .. self.lit_count as usize]
    }

    fn locked(&self, env: &env::SolverEnv) -> bool {
        let first_watched = self.literals[0];
        let reason = env.decision_reasons[core::variable(first_watched)];
        match reason {
            None => false,
            Some(dreason) => dreason.unique_id() == self.id,
        }
    }

    fn activity(&self) -> f64 {
        self.activity
    }

    fn set_activity(&mut self, new_activity : f64) -> () {
        self.activity = new_activity;
    }

    fn unique_id(&self) -> u64 {
        self.id
    }
}

const LEARNED_MASK : u16 = 0x1;

fn is_learned(cl : &Clause) -> bool {
    cl.flags & LEARNED_MASK == LEARNED_MASK
}

fn bump_clause_activity(env: &mut env::SolverEnv, cl : &mut Clause) -> () {
    if is_learned(cl) {
        cl.activity += env.constraint_increment;
        if cl.activity > env::ACTIVITY_CAP {
            env::rescale_activity(env);
        }
    }
}

fn remove_literal(cl : &mut Clause, ix : usize) -> Option<core::Literal> {
    let lc = cl.lit_count as usize;
    if ix >= lc {
        return None;
    } else if ix == lc - 1 {
        cl.lit_count -= 1;
        return None;
    } else {
        let new_lit = cl.literals[lc - 1];
        cl.lit_count -= 1;
        cl.literals[ix] = new_lit;
        return Some(new_lit);
    }
}

fn normalize_watched_literals(cl : &mut Clause, false_lit : core::Literal) -> core::Literal {
    let l0 = cl.literals[0];
    let l1 = cl.literals[1];
    if l1 == false_lit {
        return l0;
    } else {
        swap_literals(cl, 0, 1);
        return l1;
    }
}

fn swap_literals(cl : &mut Clause, ix1 : usize, ix2 : usize) -> () {
    let l1 = cl.literals[ix1];
    cl.literals[ix1] = cl.literals[ix2];
    cl.literals[ix2] = l1;
}

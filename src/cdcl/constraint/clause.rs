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
    fn remove(&self, con : &constraint::Constraint, env: &mut env::SolverEnv) -> () {
        if self.lit_count >= 1 {
            watchlist::unwatch_literal(env, con, self.literals[0]);

            if self.lit_count >= 2 {
                watchlist::unwatch_literal(env, con, self.literals[1]);
            }
        }
    }

    fn propagate(&self, env: &mut env::SolverEnv, lit : core::Literal) -> constraint::PropagationResult {
        unimplemented!();
    }

    fn simplify(&self, env: &env::SolverEnv) -> bool {
        unimplemented!();
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
        cl.activity += env.root.constraint_increment;
        if cl.activity > env::ACTIVITY_CAP {
            env::rescale_activity(env);
        }
    }
}

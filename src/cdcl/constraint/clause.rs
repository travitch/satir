use tagged;

use cdcl::core;
use cdcl::constraint;
use cdcl::env;
use cdcl::watchlist;

pub struct Clause {
    id: u64,
    activity: f64,
    lit_count: i16,
    flags: i16,
    literals: [core::Literal],
}

impl constraint::Constraint for Clause {
    fn remove(&self, env: &mut env::SolverEnv) -> () {
        if self.lit_count >= 1 {
            // watchlist::unwatch_literal(env, self.literals[0]
        }
    }

    fn propagate(&self, env: &mut env::SolverEnv, lit : core::Literal) -> constraint::PropagationResult {
        unimplemented!();
    }

    fn simplify(&self, env: &env::SolverEnv) -> bool {
        unimplemented!();
    }

    fn reason(&self, env: &env::SolverEnv, conflict_lit : Option<core::Literal>) -> Vec<core::Literal> {
        unimplemented!();
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

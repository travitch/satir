use cdcl::core;
use cdcl::env;

mod clause;

pub enum PropagationResult {
    Conflict,
    KeepWatch,
    NewWatch,
}

pub fn remove(con: &Constraint, env: &mut env::SolverEnv) -> () {
    con.remove(con, env)
}

pub trait Constraint {
    fn remove(&self, con: &Constraint, env: &mut env::SolverEnv) -> ();
    fn propagate(&self, env: &mut env::SolverEnv, core::Literal) -> PropagationResult;
    fn simplify(&self, env: &env::SolverEnv) -> bool;
    fn reason(&mut self, env: &mut env::SolverEnv, Option<core::Literal>) -> Vec<core::Literal>;
    fn locked(&self, env: &env::SolverEnv) -> bool;
    fn activity(&self) -> f64;
    fn set_activity(&mut self, f64) -> ();
    fn unique_id(&self) -> u64;
}

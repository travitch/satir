use std::any::Any;
use cdcl::core;
use cdcl::env;

mod clause;

pub enum PropagationResult {
    Conflict,
    KeepWatch,
    NewWatch,
}

pub trait Constraint {
    fn remove(&self, env: &mut env::SolverEnv) -> ();
    fn propagate(&self, env: &mut env::SolverEnv, core::Literal) -> PropagationResult;
    fn simplify(&self, env: &env::SolverEnv) -> bool;
    fn reason(&self, env: &env::SolverEnv, Option<core::Literal>) -> Vec<core::Literal>;
    fn locked(&self, env: &env::SolverEnv) -> bool;
    fn activity(&self) -> f64;
    fn set_activity(&mut self, f64) -> ();
}

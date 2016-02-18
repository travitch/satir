use cdcl::core;
use cdcl::env;

mod clause;

pub enum PropagationResult {
    Conflict,
    KeepWatch,
    NewWatch,
}

pub fn remove<'a>(con: &'a Constraint, env: &mut env::SolverEnv<'a>) -> () {
    con.remove(con, env)
}

pub trait Constraint {
    fn remove<'a>(&self, con: &'a Constraint, env: &mut env::SolverEnv<'a>) -> ();
    fn propagate<'a>(&mut self, con: &'a Constraint, env: &mut env::SolverEnv<'a>, core::Literal) -> PropagationResult;
    fn simplify<'a>(&mut self, con: &'a Constraint, env: &mut env::SolverEnv<'a>) -> bool;
    fn reason(&mut self, env: &mut env::SolverEnv, Option<core::Literal>) -> &[core::Literal];
    fn locked(&self, env: &env::SolverEnv) -> bool;
    fn activity(&self) -> f64;
    fn set_activity(&mut self, f64) -> ();
    fn unique_id(&self) -> u64;
}

/* Note [Constraint Interface]

This will have to evolve as I learn.

It isn't clear what the most useful return type for `reason` is.  The
slice is useful for clauses, but might not be possible for equality
constraints.  An iterator might be a good return type.

The `unique_id` method is unfortunate.  It is used to check for
constraint identity, which proved difficult to do directly on trait
objects (especially when one of the trait objects is unsized).  The
current setup requires care in assigning ids (see the id source in
env).  This places a limit on the total number of constraints that are
ever created.  Granted, that limit is quite high (2^64).

*/

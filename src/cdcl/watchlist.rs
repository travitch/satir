use cdcl::constraint;
use cdcl::core;
use cdcl::env;

use tagged;

pub fn unwatch_literal(env : &mut env::SolverEnv, con : &constraint::Constraint, lit : core::Literal) -> () {
    
}

pub fn watch_literal<'a>(env : &mut env::SolverEnv<'a>, con : &'a constraint::Constraint, lit : core::Literal) -> () {
    env.watchlist[lit].push(con);
}


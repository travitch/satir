use cdcl::constraint;
use cdcl::core;
use cdcl::env;

use tagged;

pub fn unwatch_literal(env : &mut env::SolverEnv, con : &constraint::Constraint, lit : core::Literal) -> () {
    let ref mut watchers = &mut env.watchlist[lit];
    let mut rem_ix = 0;
    let mut found_con = false;
    for (ix, constraint) in watchers.iter().enumerate() {
        if constraint.unique_id() == con.unique_id() {
            rem_ix = ix;
            found_con = true;
            break;
        }
    }

    if !found_con {
        panic!("No constraint found in unwatch_literal");
    }

    watchers.swap_remove(rem_ix);
}

pub fn watch_literal<'a>(env : &mut env::SolverEnv<'a>, con : &'a constraint::Constraint, lit : core::Literal) -> () {
    env.watchlist[lit].push(con);
}


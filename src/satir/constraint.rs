use crate::satir::core;

/// The possible results of propagation
pub enum PropagationResult {
    Conflict,
    KeepWatch,
    NewWatch,
}


/// Constraints are a generalization of clauses, and represent any learned or
/// stated constraint that must be satisfied
pub trait Constraint {
    /// Get the unique identifier of the constraint
    fn unique_id(&self) -> u64;
    /// Get the activity of the constraint (for use in heuristics to determine which constraints to keep or discard)
    fn activity(&self) -> f64;
    /// Propagate units to look for conflicts
    ///
    /// This can mutate self as it is allowed to modify constraint state
    // fn propagate(&mut self, c : dyn Constraint, l : core::Literal) -> PropagationResult;
    fn remove(&mut self, c : &dyn Constraint);
}

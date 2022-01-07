use slice_dst;

use crate::satir::core::{Variable, Value, Literal};
use crate::satir::constraint;
use crate::satir::tagged::TaggedVec;

/// Fixed-length clause metadata
pub struct ClauseHeader {
    pub id : u64,
    pub lit_count : usize,
    pub activity : f64
}

/// A SAT clause
///
/// Note that the literals are inline in the object, which means it is not
/// sized. As a result, clauses must be stored in a box. However, we need to do
/// that anyway because all constraints must be trait objects.
///
/// Since the `Box` is always in this structure, we are able to have a Vec<Clause>
///
/// The key invariant of the `Clause` data type is that the first two literals
/// are the watched literals.  Clauses with fewer than two literals are removed
/// during preprocessing.
pub struct Clause(Box<slice_dst::SliceWithHeader<ClauseHeader, Literal>>);

pub enum PropagateResult {
    Conflict,
    NoConflict
}

fn lit_value(assignment : &TaggedVec<Variable, Value>, lit : &Literal) -> Value {
    assignment[lit.variable()]
}

impl Clause {
    pub fn new<I>(head : ClauseHeader, lits : I) -> Self
    where
        I : IntoIterator<Item = Literal>,
        I::IntoIter : ExactSizeIterator,
    {
        Clause(slice_dst::SliceWithHeader::new::<Box<_>, I>(head, lits).into())
    }

    /// The number of active literals (i.e., non-deleted literals)
    pub fn lit_count(&self) -> usize {
        self.0.header.lit_count
    }

    pub fn propatate_units(&self, assignment : &TaggedVec<Variable, Value>, lit : &Literal) -> PropagateResult {
        // There is a conflict if all of the literals in the clause evaluate to
        // False
        //
        // If *any* literal is either True or Unassigned, there is no conflict yet
        for idx in 0..self.lit_count() {
            let val = self[idx].under_value(assignment[self[idx].variable()]);
            if val == Value::LIFTED_FALSE {
                continue;
            }

            return PropagateResult::NoConflict;
        }

        return PropagateResult::Conflict;
    }
}

// Note: Morally, `Clause` is this type:
//
// pub struct Clause {
//     id : u64,
//     lit_count : u16,
//     activity : f64,
//     literals : [core::Literal]
// }
//
// However, allocating a Dynamically-sized Type (DST) in Rust is unsafe and
// difficult, so we use slice_dst to do the heavy lifting for us. It ensures
// that the header and slice are inlined in each object

impl std::ops::Index<usize> for Clause {
    type Output = Literal;

    fn index(&self, num : usize) -> &Self::Output {
        &self.0.slice[num]
    }
}

impl std::ops::IndexMut<usize> for Clause {
    fn index_mut<'a>(&'a mut self, i : usize) -> &'a mut Literal {
        &mut self.0.slice[i]
    }
}

impl constraint::Constraint for Clause {
    fn unique_id(&self) -> u64 {
        self.0.header.id
    }

    fn activity(&self) -> f64 {
        self.0.header.activity
    }

    fn remove(&mut self, c : &dyn constraint::Constraint) {
        if self.0.header.lit_count >= 1 {

        }
    }
}

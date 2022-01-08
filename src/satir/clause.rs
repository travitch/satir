use slice_dst;

use crate::satir::core::{Variable, Value, Literal};
use crate::satir::tagged::{TaggedIndexable, TaggedVec};

/// The unique identifier for a clause
///
/// This is intended to be the index into the clause array that holds the
/// `Clause`.  We need these indirect references because we can't have
/// references to clauses (since we need to borrow them mutably in many places).
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClauseId(pub i64);

impl TaggedIndexable for ClauseId {
    fn as_index(&self) -> usize {
        let ClauseId(i) = self;
        *i as usize
    }
}

/// Fixed-length clause metadata
pub struct ClauseHeader {
    pub id : ClauseId,
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

    pub fn identifier(&self) -> ClauseId {
        self.0.header.id
    }

    pub fn identifier_mut(&mut self) -> &mut ClauseId {
        &mut self.0.header.id
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

// impl constraint::Constraint for Clause {
//     fn unique_id(&self) -> ClauseId {
//         self.0.header.id
//     }

//     fn activity(&self) -> f64 {
//         self.0.header.activity
//     }

//     fn remove(&mut self, c : &dyn constraint::Constraint) {
//         if self.0.header.lit_count >= 1 {

//         }
//     }
// }

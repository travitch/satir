use std::marker::PhantomData;

pub struct Variable(i32);
pub struct Literal(i32);
pub struct Value(i8);

// Variables

pub const FIRST_VARIABLE : Variable = Variable(0);

pub fn next_variable(v : Variable) -> Variable {
    let Variable(vnum) = v;
    Variable(vnum + 1)
}

pub fn previous_variable(v : Variable) -> Option<Variable> {
    let Variable(vnum) = v;
    if vnum <= 0 {
        None
    } else {
        Some(Variable(vnum - 1))
    }
}

pub fn to_positive_literal(v : Variable) -> Literal {
    let Variable(vnum) = v;
    Literal(vnum << 1)
}

pub fn to_negative_literal(v : Variable) -> Literal {
    let Variable(vnum) = v;
    Literal((vnum << 1) | 1)
}

// Literals

pub fn negate_literal(l : Literal) -> Literal {
    let Literal(lnum) = l;
    Literal(lnum ^ 1)
}

pub fn is_negated(l : Literal) -> bool {
    let Literal(lnum) = l;
    lnum & 1 == 0
}

pub fn variable(l : Literal) -> Variable {
    let Literal(lnum) = l;
    Variable(lnum >> 1)
}

// Type-safe array indexing

pub struct TaggedArray<'a,I,T : 'a> {
    index_type: PhantomData<I>,
    tagged_array: &'a [T],
}

pub trait TaggedIndexable {
    fn tag_index(&self) -> usize;
}

pub fn tagged_index<'a, I, T>(arr : &TaggedArray<'a, I, T>, ix : I) -> T
    where I : TaggedIndexable, T : Copy {
    arr.tagged_array[ix.tag_index()]
}

// Clauses

pub struct Clause {
    activity: f64,
    lit_count: i16,
    flags: i16,
    literals: [Literal],
}

pub struct SolverEnv<'a> {
    assignment: TaggedArray<'a,Variable, Value>,
    variable_levels: TaggedArray<'a,Variable, i32>,
    variable_activity: TaggedArray<'a,Variable, f64>,
//    decision_reasons: TaggedArray<'a,Variable, Option<'a &Clause>>,
}

#[test]
fn it_works() {
}

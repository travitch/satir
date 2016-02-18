use tagged;

#[derive(Clone,Copy)]
pub struct Variable(i32);
#[derive(Clone,Copy)]
pub struct Literal(i32);
#[derive(Clone,Copy,PartialOrd,PartialEq)]
pub struct Value(i8);

impl tagged::TaggedIndexable for Variable {
    fn as_index(&self) -> usize {
        let &Variable(vnum) = self;
        vnum as usize
    }
}

impl tagged::TaggedIndexable for Literal {
    fn as_index(&self) -> usize {
        let &Literal(lnum) = self;
        lnum as usize
    }
}

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

pub fn lit_val(l : Literal, v : Value) -> Value {
    let Literal(lval) = l;
    let Value(val) = v;
    Value(val ^ ((lval & 1) as i8))
}

pub const LIFTED_FALSE : Value = Value(1);
pub const LIFTED_TRUE : Value = Value(0);
pub const UNASSIGNED : Value = Value(2);

pub fn is_unassigned(v : Value) -> bool {
    v >= UNASSIGNED
}

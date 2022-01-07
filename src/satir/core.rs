use crate::satir::tagged;

#[derive(Clone,Copy,Debug,Eq,Ord,PartialEq,PartialOrd,Hash)]
pub struct Variable(i32);

#[derive(Clone,Copy,Debug,Eq,Ord,PartialEq,PartialOrd)]
pub struct Literal(i32);

/// Values are True, False, or Unassigned
#[derive(Clone,Copy,Debug,Eq,Ord,PartialEq,PartialOrd)]
pub struct Value(i8);

/// Enable `Variable`s to be used as indexes into appropriately typed structures
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

impl Variable {
    pub const FIRST_VARIABLE : Variable = Variable(0);

    pub fn next_variable(&self) -> Variable {
        let Variable(num) = self;
        Variable(num + 1)
    }

    pub fn to_positive_literal(&self) -> Literal {
        let Variable(vnum) = self;
        Literal(vnum << 1)
    }

    pub fn to_negative_literal(&self) -> Literal {
        let Variable(vnum) = self;
        Literal((vnum << 1) | 1)
    }
}

impl Literal {
    pub fn variable(&self) -> Variable {
        let Literal(lnum) = self;
        Variable(lnum >> 1)
    }

    pub fn is_negated(&self) -> bool {
        let Literal(lnum) = self;
        lnum & 1 == 0
    }

    pub fn negate(&self) -> Literal {
        let Literal(lnum) = self;
        Literal(lnum ^ 1)
    }

    pub fn under_value(&self, v : Value) -> Value {
        let Literal(lval) = self;
        let Value(val) = v;
        Value(val ^ ((lval & 1) as i8))
    }

    pub fn satisfy(&self) -> Value {
        let Literal(lval) = self;
        Value((lval & 1) as i8)
    }
}

impl Value {
    pub const LIFTED_FALSE : Value = Value(1);
    pub const LIFTED_TRUE : Value = Value(0);
    pub const UNASSIGNED : Value = Value(2);

    pub fn is_unassigned(&self) -> bool {
        *self >= Value::UNASSIGNED
    }
}


pub enum Result {
    Unsat,
    Sat
}

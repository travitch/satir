use std::marker::PhantomData;

// Type-safe array indexing

pub trait TaggedIndexable {
    fn as_index(&self) -> usize;
}

pub fn tagged_index<I, T>(arr : &TaggedVec<I, T>, ix : I) -> T
    where I : TaggedIndexable, T : Copy {
    arr.tagged_vec[ix.as_index()]
}

pub struct TaggedVec<I,T> {
    index_type: PhantomData<I>,
    tagged_vec: Vec<T>,
}

use std::marker::PhantomData;

// Type-safe array indexing

pub struct TaggedArray<I,T> {
    index_type: PhantomData<I>,
    tagged_array: [T],
}

pub trait TaggedIndexable {
    fn tag_index(&self) -> usize;
}

pub fn tagged_index<I, T>(arr : &TaggedArray<I, T>, ix : I) -> T
    where I : TaggedIndexable, T : Copy {
    arr.tagged_array[ix.tag_index()]
}

pub struct TaggedVec<I,T> {
    index_type: PhantomData<I>,
    tagged_vec: Vec<T>,
}

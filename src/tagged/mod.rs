use std::marker::PhantomData;

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

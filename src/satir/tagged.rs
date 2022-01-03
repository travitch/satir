use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

// Type-safe array indexing

pub trait TaggedIndexable {
    fn as_index(&self) -> usize;
}

impl<I : TaggedIndexable, T> Index<I> for TaggedVec<I, T> {
    type Output = T;
    fn index<'a>(&'a self, i : I) -> &'a T {
        &self.tagged_vec[i.as_index()]
    }
}

impl<I : TaggedIndexable, T> IndexMut<I> for TaggedVec<I, T> {
    fn index_mut<'a>(&'a mut self, i : I) -> &'a mut T {
        &mut self.tagged_vec[i.as_index()]
    }
}

pub fn tagged_index<I, T>(arr : &TaggedVec<I, T>, ix : I) -> T
    where I : TaggedIndexable, T : Copy {
    arr.tagged_vec[ix.as_index()]
}

pub fn tagged_index_ref<I, T>(arr : &mut TaggedVec<I, T>, ix : I) -> &mut T
    where I : TaggedIndexable {
    // &arr.tagged_vec[ix.as_index()]
    unimplemented!()
}

pub struct TaggedVec<I,T> {
    index_type: PhantomData<I>,
    tagged_vec: Vec<T>,
}

impl<I,T> TaggedVec<I,T> {
    pub fn new() -> Self {
        TaggedVec {
            index_type : PhantomData::default(),
            tagged_vec : Vec::new()
        }
    }

    pub fn push(&mut self, t: T) -> () {
        self.tagged_vec.push(t);
    }

    pub fn len(&self) -> usize {
        self.tagged_vec.len()
    }

    /// Ensure that the underlying `Vec` has enough storage to hold the given
    /// index value, extending the underlying storage (with a default value) if
    /// necessary
    pub fn ensure_index(&mut self, i : &I, t: T) -> ()
    where
        T : Clone,
        I : TaggedIndexable
    {
        if self.tagged_vec.len() < i.as_index() {
            self.tagged_vec.resize(i.as_index(), t);
        }
    }
}

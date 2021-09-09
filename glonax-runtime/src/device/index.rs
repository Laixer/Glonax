pub trait TryIndex<Idx: ?Sized> {
    type Output;

    /// Performs the try indexing operation.
    ///
    /// Returns `None` if the index is out of bounds.
    fn try_index(&self, index: Idx) -> Option<&Self::Output>;
}

pub trait TryIndexMut<Idx: ?Sized>: TryIndex<Idx> {
    /// Performs the mutable try indexing operation.
    ///
    /// Returns `None` if the index is out of bounds.
    fn try_index_mut(&mut self, index: Idx) -> Option<&mut Self::Output>;
}

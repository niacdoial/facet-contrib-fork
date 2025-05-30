use super::Peek;
use facet_core::ListDef;

/// Iterator over a `PeekList`
pub struct PeekListIter<'mem> {
    list: PeekList<'mem>,
    index: usize,
    len: usize,
}

impl<'mem> Iterator for PeekListIter<'mem> {
    type Item = Peek<'mem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        let item = self.list.get(self.index);
        self.index += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for PeekListIter<'_> {}

impl<'mem> IntoIterator for &'mem PeekList<'mem> {
    type Item = Peek<'mem>;
    type IntoIter = PeekListIter<'mem>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Lets you read from a list (implements read-only [`facet_core::ListVTable`] proxies)
#[derive(Clone, Copy)]
pub struct PeekList<'mem> {
    pub(crate) value: Peek<'mem>,
    pub(crate) def: ListDef,
}

impl<'mem> PeekList<'mem> {
    /// Creates a new peek list
    pub fn new(value: Peek<'mem>, def: ListDef) -> Self {
        Self { value, def }
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        unsafe { (self.def.vtable.len)(self.value.data()) }
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get an item from the list at the specified index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    pub fn get(&self, index: usize) -> Option<Peek<'mem>> {
        if index >= self.len() {
            return None;
        }

        let item_ptr = unsafe { (self.def.vtable.get_item_ptr)(self.value.data(), index) };
        Some(Peek {
            data: item_ptr,
            shape: self.def.t(),
        })
    }

    /// Returns an iterator over the list
    pub fn iter(self) -> PeekListIter<'mem> {
        PeekListIter {
            list: self,
            index: 0,
            len: self.len(),
        }
    }

    /// Def getter
    pub fn def(&self) -> ListDef {
        self.def
    }
}

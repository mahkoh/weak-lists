use {
    crate::sync::{Iter, SyncWeakList, WeakListData},
    alloc::sync::Arc,
    core::fmt::{Debug, Formatter},
    parking_lot::Mutex,
};

impl<T> SyncWeakList<T>
where
    T: ?Sized,
{
    /// Removes all elements from the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use weak_lists::{SyncWeakList, SyncWeakListElement};
    ///
    /// let list = SyncWeakList::default();
    /// let entry = Arc::new(1);
    /// let entry = SyncWeakListElement::new(Arc::downgrade(&entry));
    /// entry.attach(&list);
    /// assert!(list.iter().next().is_some());
    /// list.clear();
    /// assert!(list.iter().next().is_none());
    /// ```
    pub fn clear(&self) {
        let data = &mut *self.data.lock();
        data.members.clear();
    }

    /// Creates an iterator over the entries of the list.
    ///
    /// The list can be mutated during the iteration. It is guaranteed that, if an element
    /// was part of the list when this iterator was created, and if the element was not
    /// removed during the iteration, then the element will be returned exactly once by
    /// this iterator.
    pub fn iter(&self) -> Iter<'_, T> {
        let data = &mut *self.data.lock();
        if data.active_iterators == 0 {
            data.members.compact();
        }
        data.active_iterators += 1;
        Iter {
            iter: 0..data.members.index_len(),
            data: &self.data,
        }
    }
}

impl<T> Default for SyncWeakList<T>
where
    T: ?Sized,
{
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(WeakListData {
                next_id: 0,
                active_iterators: 0,
                members: Default::default(),
            })),
        }
    }
}

impl<'a, T> IntoIterator for &'a SyncWeakList<T>
where
    T: ?Sized,
{
    type Item = Arc<T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Debug for SyncWeakList<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SyncWeakList")
            .field("id", &Arc::as_ptr(&self.data))
            .finish_non_exhaustive()
    }
}

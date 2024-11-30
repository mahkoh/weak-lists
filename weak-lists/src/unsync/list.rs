use {
    crate::unsync::{Iter, WeakList, WeakListData},
    alloc::rc::Rc,
    core::{
        cell::UnsafeCell,
        fmt::{Debug, Formatter},
    },
};

impl<T> WeakList<T>
where
    T: ?Sized,
{
    /// Removes all elements from the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::{Rc, Weak};
    /// use weak_lists::{WeakList, WeakListElement};
    ///
    /// let list = WeakList::default();
    /// let entry = Rc::new(1);
    /// let entry = WeakListElement::new(Rc::downgrade(&entry));
    /// entry.attach(&list);
    /// assert!(list.iter().next().is_some());
    /// list.clear();
    /// assert!(list.iter().next().is_none());
    /// ```
    pub fn clear(&self) {
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            // - In particular, dropping the Weak objects below will never run the drop
            //   impl of T itself.
            &mut *self.data.get()
        };
        data.members.clear();
    }

    /// Creates an iterator over the entries of the list.
    ///
    /// The list can be mutated during the iteration. It is guaranteed that, if an element
    /// was part of the list when this iterator was created, and if the element was not
    /// removed during the iteration, then the element will be returned exactly once by
    /// this iterator.
    pub fn iter(&self) -> Iter<'_, T> {
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            // - In particular, the calls to compact and index_len are safe.
            &mut *self.data.get()
        };
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

impl<T> Default for WeakList<T>
where
    T: ?Sized,
{
    fn default() -> Self {
        Self {
            data: Rc::new(UnsafeCell::new(WeakListData {
                next_id: 0,
                active_iterators: 0,
                members: Default::default(),
            })),
        }
    }
}

impl<'a, T> IntoIterator for &'a WeakList<T>
where
    T: ?Sized,
{
    type Item = Rc<T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Debug for WeakList<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WeakList")
            .field("id", &Rc::as_ptr(&self.data))
            .finish_non_exhaustive()
    }
}

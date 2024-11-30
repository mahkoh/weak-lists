use {
    crate::unsync::{EntryData, WeakList, WeakListElement},
    alloc::rc::{Rc, Weak},
    core::{
        cell::UnsafeCell,
        fmt::{Debug, Formatter},
        mem,
    },
};

impl<T> WeakListElement<T>
where
    T: ?Sized,
{
    /// Creates a new list element.
    ///
    /// This object holds a weak reference to the `T`. When this object is dropped, it
    /// automatically detaches itself from the list it is currently attached to.
    ///
    /// Often, this object is directly contained in the `T`:
    ///
    /// ```rust
    /// use std::rc::Rc;
    /// use weak_lists::WeakListElement;
    ///
    /// struct Client {
    ///     element: WeakListElement<Client>,
    /// }
    ///
    /// let client = Rc::new_cyclic(|slf| Client {
    ///     element: WeakListElement::new(slf.clone()),
    /// });
    /// ```
    ///
    /// Since only weak references are stored, this does not create any actual reference
    /// cycles.
    pub fn new(t: Weak<T>) -> Self {
        Self {
            t,
            data: UnsafeCell::new(EntryData {
                id: 0,
                owner: Default::default(),
            }),
        }
    }

    /// Attaches the list element to a list.
    ///
    /// If this object was previously attached to a list, it is automatically detached
    /// from that list.
    ///
    /// The list will only hold on a weak reference to this element and vice versa.
    ///
    /// Any existing iterator over the list might or might not see this element, this is
    /// unspecified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::rc::Rc;
    /// use weak_lists::{WeakList, WeakListElement};
    ///
    /// struct Client {
    ///     element: WeakListElement<Client>,
    /// }
    ///
    /// let clients1 = WeakList::default();
    ///
    /// let client = Rc::new_cyclic(|slf| Client {
    ///     element: WeakListElement::new(slf.clone()),
    /// });
    ///
    /// client.element.attach(&clients1);
    ///
    /// assert!(clients1.iter().next().is_some());
    ///
    /// let clients2 = WeakList::default();
    ///
    /// client.element.attach(&clients2);
    ///
    /// assert!(clients1.iter().next().is_none());
    /// assert!(clients2.iter().next().is_some());
    /// ```
    pub fn attach(&self, to: &WeakList<T>) {
        self.detach();
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            // - In particular, the clone call below clones an Rc and is therefore safe.
            // - The insert call only adds an element to a map and is therefore safe.
            // - list_data.next_id cannot overflow, therefore the insert call returns none
            //   and no drop code runs. But even if it did run, it would run after all
            //   uses of the mutable references have concluded.
            &mut *self.data.get()
        };
        data.owner = Rc::downgrade(&to.data);
        let list_data = unsafe {
            // SAFETY: See the previous safety comment.
            &mut *to.data.get()
        };
        data.id = list_data.next_id;
        list_data.next_id += 1;
        list_data.members.insert(data.id, self.t.clone());
    }

    /// Detaches the element from its current list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::rc::Rc;
    /// use weak_lists::{WeakList, WeakListElement};
    ///
    /// struct Client {
    ///     element: WeakListElement<Client>,
    /// }
    ///
    /// let clients = WeakList::default();
    ///
    /// let client = Rc::new_cyclic(|slf| Client {
    ///     element: WeakListElement::new(slf.clone()),
    /// });
    ///
    /// client.element.attach(&clients);
    ///
    /// assert!(clients.iter().next().is_some());
    ///
    /// client.element.detach();
    ///
    /// assert!(clients.iter().next().is_none());
    /// ```
    pub fn detach(&self) {
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            // - All drop code below runs after the last use of the references has
            //   concluded. However, even if it did run, it could be shown that that code
            //   is harmless and does not run any code that depends on T.
            &mut *self.data.get()
        };
        let prev = mem::take(&mut data.owner).upgrade();
        if let Some(prev) = prev {
            let list_data = unsafe {
                // SAFETY: See the previous safety comment.
                &mut *prev.get()
            };
            list_data.members.remove(&data.id);
        }
    }
}

impl<T> Drop for WeakListElement<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.detach();
    }
}

impl<T> Debug for WeakListElement<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            &mut *self.data.get()
        };
        let owner = data.owner.upgrade();
        let owner_id = owner.as_ref().map(Rc::as_ptr);
        f.debug_struct("WeakListElement")
            .field("list", &owner_id)
            .finish_non_exhaustive()
    }
}

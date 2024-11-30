use {
    crate::sync::{EntryData, SyncWeakList, SyncWeakListElement},
    alloc::sync::{Arc, Weak},
    core::{
        fmt::{Debug, Formatter},
        mem,
    },
    parking_lot::Mutex,
};

impl<T> SyncWeakListElement<T>
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
    /// use std::sync::Arc;
    /// use weak_lists::SyncWeakListElement;
    ///
    /// struct Client {
    ///     element: SyncWeakListElement<Client>,
    /// }
    ///
    /// let client = Arc::new_cyclic(|slf| Client {
    ///     element: SyncWeakListElement::new(slf.clone()),
    /// });
    /// ```
    ///
    /// Since only weak references are stored, this does not create any actual reference
    /// cycles.
    pub fn new(t: Weak<T>) -> Self {
        Self {
            t,
            data: Mutex::new(EntryData {
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
    /// use std::sync::Arc;
    /// use weak_lists::{SyncWeakList, SyncWeakListElement};
    ///
    /// struct Client {
    ///     element: SyncWeakListElement<Client>,
    /// }
    ///
    /// let clients1 = SyncWeakList::default();
    ///
    /// let client = Arc::new_cyclic(|slf| Client {
    ///     element: SyncWeakListElement::new(slf.clone()),
    /// });
    ///
    /// client.element.attach(&clients1);
    ///
    /// assert!(clients1.iter().next().is_some());
    ///
    /// let clients2 = SyncWeakList::default();
    ///
    /// client.element.attach(&clients2);
    ///
    /// assert!(clients1.iter().next().is_none());
    /// assert!(clients2.iter().next().is_some());
    /// ```
    pub fn attach(&self, to: &SyncWeakList<T>) {
        self.detach();
        let data = &mut *self.data.lock();
        data.owner = Arc::downgrade(&to.data);
        let list_data = &mut *to.data.lock();
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
    /// use std::sync::Arc;
    /// use weak_lists::{SyncWeakList, SyncWeakListElement};
    ///
    /// struct Client {
    ///     element: SyncWeakListElement<Client>,
    /// }
    ///
    /// let clients = SyncWeakList::default();
    ///
    /// let client = Arc::new_cyclic(|slf| Client {
    ///     element: SyncWeakListElement::new(slf.clone()),
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
        let data = &mut *self.data.lock();
        let prev = mem::take(&mut data.owner).upgrade();
        if let Some(prev) = prev {
            let list_data = &mut *prev.lock();
            list_data.members.remove(&data.id);
        }
    }
}

impl<T> Drop for SyncWeakListElement<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.detach();
    }
}

impl<T> Debug for SyncWeakListElement<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let data = self.data.lock();
        let owner = data.owner.upgrade();
        let owner_id = owner.as_ref().map(Arc::as_ptr);
        f.debug_struct("SyncWeakListElement")
            .field("list", &owner_id)
            .finish_non_exhaustive()
    }
}

//! The thread-safe version of the list.

mod element;
mod iter;
mod list;
#[cfg(test)]
mod tests;

use {
    alloc::sync::{Arc, Weak},
    core::ops::Range,
    parking_lot::Mutex,
    stable_map::StableMap,
};

/// A thread-safe list holding weak references to its elements.
///
/// The list does not hold strong references to its elements and the elements do not hold
/// strong references to the list. You must use some other mechanism to keep all parties
/// alive.
///
/// This list supports concurrent iteration and modification.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use {
///     weak_lists::{SyncWeakList, SyncWeakListElement},
///     std::{
///         array,
///     },
/// };
///
/// pub struct Service {
///     callbacks: SyncWeakList<dyn Callback>,
/// }
///
/// pub trait Callback {
///     fn run(&self);
/// }
///
/// impl Service {
///     pub fn register_callback(&self, callback: &SyncWeakListElement<dyn Callback>) {
///         callback.attach(&self.callbacks);
///     }
///
///     pub fn run_callbacks(&self) {
///         for callback in &self.callbacks {
///             callback.run();
///         }
///     }
/// }
///
/// struct Client {
///     id: usize,
///     entry: SyncWeakListElement<dyn Callback>,
/// }
///
/// impl Callback for Client {
///     fn run(&self) {
///         eprintln!("Callback {} invoked", self.id);
///         if self.id == 1 {
///             self.entry.detach();
///         }
///     }
/// }
///
/// let service = Service {
///     callbacks: Default::default(),
/// };
/// let clients = array::from_fn::<_, 3, _>(|id| {
///     Arc::<Client>::new_cyclic(|slf| Client {
///         id,
///         entry: SyncWeakListElement::new(slf.clone()),
///     })
/// });
/// for client in &clients {
///     service.register_callback(&client.entry);
/// }
/// service.run_callbacks();
/// // Callback 0 invoked
/// // Callback 1 invoked
/// // Callback 2 invoked
/// service.run_callbacks();
/// // Callback 0 invoked
/// // Callback 2 invoked
/// ```
pub struct SyncWeakList<T>
where
    T: ?Sized,
{
    data: Arc<Mutex<WeakListData<T>>>,
}

struct WeakListData<T>
where
    T: ?Sized,
{
    next_id: u64,
    active_iterators: usize,
    members: StableMap<u64, Weak<T>>,
}

/// An thread-safe element that can be inserted into a weak list.
///
/// Each element can be attached to 0 or 1 list. Attaching it to a list automatically
/// detaches itself from the previous list.
///
/// When this object is dropped, it detaches itself from its current list.
pub struct SyncWeakListElement<T>
where
    T: ?Sized,
{
    t: Weak<T>,
    data: Mutex<EntryData<T>>,
}

struct EntryData<T>
where
    T: ?Sized,
{
    id: u64,
    owner: Weak<Mutex<WeakListData<T>>>,
}

/// An iterator over list elements.
///
/// This object is created by calling [iter](SyncWeakList::iter) or by using the
/// [IntoIterator] implementation of `&SyncWeakList`.
pub struct Iter<'a, T>
where
    T: ?Sized,
{
    iter: Range<usize>,
    data: &'a Mutex<WeakListData<T>>,
}

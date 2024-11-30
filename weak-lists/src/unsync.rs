//! The single-threaded version of the list.

mod element;
mod iter;
mod list;
#[cfg(test)]
mod tests;

use {
    alloc::rc::{Rc, Weak},
    core::{cell::UnsafeCell, ops::Range},
    stable_map::StableMap,
};

/// A list holding weak references to its elements.
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
/// use {
///     weak_lists::{WeakList, WeakListElement},
///     std::{
///         array,
///         rc::{Rc, Weak},
///     },
/// };
///
/// pub struct Service {
///     callbacks: WeakList<dyn Callback>,
/// }
///
/// pub trait Callback {
///     fn run(&self);
/// }
///
/// impl Service {
///     pub fn register_callback(&self, callback: &WeakListElement<dyn Callback>) {
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
///     entry: WeakListElement<dyn Callback>,
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
///     Rc::<Client>::new_cyclic(|slf| Client {
///         id,
///         entry: WeakListElement::new(slf.clone()),
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
pub struct WeakList<T>
where
    T: ?Sized,
{
    data: Rc<UnsafeCell<WeakListData<T>>>,
}

struct WeakListData<T>
where
    T: ?Sized,
{
    next_id: u64,
    active_iterators: usize,
    members: StableMap<u64, Weak<T>>,
}

/// An element that can be inserted into a weak list.
///
/// Each element can be attached to 0 or 1 list. Attaching it to a list automatically
/// detaches itself from the previous list.
///
/// When this object is dropped, it detaches itself from its current list.
pub struct WeakListElement<T>
where
    T: ?Sized,
{
    t: Weak<T>,
    data: UnsafeCell<EntryData<T>>,
}

struct EntryData<T>
where
    T: ?Sized,
{
    id: u64,
    owner: Weak<UnsafeCell<WeakListData<T>>>,
}

/// An iterator over list elements.
///
/// This object is created by calling [iter](WeakList::iter) or by using the
/// [IntoIterator] implementation of `&WeakList`.
pub struct Iter<'a, T>
where
    T: ?Sized,
{
    iter: Range<usize>,
    data: &'a UnsafeCell<WeakListData<T>>,
}

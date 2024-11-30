use {
    crate::unsync::Iter,
    alloc::rc::Rc,
    core::{
        fmt::{Debug, Formatter},
        iter::FusedIterator,
    },
};

impl<T> Drop for Iter<'_, T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        let data = unsafe { &mut *self.data.get() };
        data.active_iterators -= 1;
    }
}

impl<T> Iterator for Iter<'_, T>
where
    T: ?Sized,
{
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        for idx in self.iter.by_ref() {
            let data = unsafe {
                // SAFETY:
                // - While we hold this reference, we do not call any functions that might
                //   create additional references to self.data. This applies to all code that
                //   creates references to self.data.
                // - Therefore, this is an exclusive reference to self.data.
                // - The get_by_index and upgrade calls below only run well-known code
                //   that does not depend on T.
                &mut *self.data.get()
            };
            if let Some(member) = data.members.get_by_index(idx) {
                if let Some(member) = member.upgrade() {
                    return Some(member);
                }
            }
        }
        None
    }
}

impl<T> Clone for Iter<'_, T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        let data = unsafe {
            // SAFETY:
            // - While we hold this reference, we do not call any functions that might
            //   create additional references to self.data. This applies to all code that
            //   creates references to self.data.
            // - Therefore, this is an exclusive reference to self.data.
            &mut *self.data.get()
        };
        data.active_iterators += 1;
        Self {
            iter: self.iter.clone(),
            data: self.data,
        }
    }
}

impl<T> Debug for Iter<'_, T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<T> FusedIterator for Iter<'_, T> where T: ?Sized {}

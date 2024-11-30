use {
    crate::sync::Iter,
    alloc::sync::Arc,
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
        let data = &mut *self.data.lock();
        data.active_iterators -= 1;
    }
}

impl<T> Iterator for Iter<'_, T>
where
    T: ?Sized,
{
    type Item = Arc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        for idx in self.iter.by_ref() {
            let data = self.data.lock();
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
        let data = &mut *self.data.lock();
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

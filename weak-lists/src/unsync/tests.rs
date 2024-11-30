use {
    crate::unsync::{WeakList, WeakListElement},
    alloc::rc::Rc,
    core::array,
};

#[derive(Debug)]
struct Element {
    i: usize,
    element: WeakListElement<Element>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i
    }
}

impl Element {
    fn new(i: usize) -> Rc<Self> {
        Rc::new_cyclic(|slf| Self {
            i,
            element: WeakListElement::new(slf.clone()),
        })
    }
}

#[test]
fn clear() {
    let list = WeakList::default();
    let entry = Element::new(0);
    entry.element.attach(&list);
    assert!(list.iter().next().is_some());
    list.clear();
    assert!(list.iter().next().is_none());
}

#[test]
fn attach_detach() {
    let list = WeakList::<Element>::default();
    let entries: [_; 3] = array::from_fn(|i| Element::new(1 << i));
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 0);
    entries[0].element.attach(&list);
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 1);
    entries[1].element.attach(&list);
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 3);
    entries[2].element.attach(&list);
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 7);
    entries[1].element.detach();
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 5);
    assert_eq!(
        list.iter()
            .map(|e| e.i)
            .inspect(|i| {
                if *i == 1 {
                    entries[1].element.attach(&list)
                }
            })
            .sum::<usize>(),
        7
    );
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 7);
    entries[0].element.detach();
    assert_eq!(
        list.iter()
            .map(|e| e.i)
            .inspect(|i| {
                if *i == 2 {
                    entries[0].element.attach(&list)
                }
            })
            .sum::<usize>(),
        6
    );
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 7);
    assert_eq!(
        list.iter()
            .map(|e| e.i)
            .inspect(|i| {
                if *i == 1 {
                    entries[1].element.detach();
                    entries[2].element.detach();
                }
            })
            .sum::<usize>(),
        1
    );
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 1);
}

#[test]
fn no_compact_with_iter() {
    let list = WeakList::<Element>::default();
    let entries: [_; 16] = array::from_fn(|i| Element::new(1 << i));
    for entry in &entries {
        entry.element.attach(&list);
    }
    for i in 0..15 {
        entries[i].element.detach();
    }
    let mut iter = list.iter();
    entries[0].element.attach(&list);
    assert_eq!(iter.next().unwrap().i, 1 << 15);
    assert!(iter.next().is_none());
    list.clear();
    for entry in &entries {
        entry.element.attach(&list);
    }
    for i in 0..15 {
        entries[i].element.detach();
    }
    let mut iter = list.iter();
    entries[0].element.attach(&list);
    assert_eq!(iter.next().unwrap().i, 1 << 0);
    assert_eq!(iter.next().unwrap().i, 1 << 15);
}

#[test]
fn clone_iter() {
    let list = WeakList::<Element>::default();
    let entries: [_; 3] = array::from_fn(|i| Element::new(1 << i));
    for entry in &entries {
        entry.element.attach(&list);
    }
    let mut iter1 = list.iter();
    iter1.next();
    let mut iter2 = iter1.clone();
    assert_eq!(iter1.next(), iter2.next());
    assert_eq!(iter1.next(), iter2.next());
    assert_eq!(iter1.next(), iter2.next());
}

#[test]
fn into_iter() {
    let list = WeakList::<Element>::default();
    let entries: [_; 3] = array::from_fn(|i| Element::new(1 << i));
    for entry in &entries {
        entry.element.attach(&list);
    }
    let mut iter1 = (&list).into_iter();
    let mut iter2 = list.iter();
    assert_eq!(iter1.next(), iter2.next());
    assert_eq!(iter1.next(), iter2.next());
    assert_eq!(iter1.next(), iter2.next());
    assert_eq!(iter1.next(), iter2.next());
}

#[test]
fn detach_on_drop() {
    let list = WeakList::<Element>::default();
    let entry = Element::new(1);
    entry.element.attach(&list);
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 1);
    drop(entry);
    assert_eq!(list.iter().map(|e| e.i).sum::<usize>(), 0);
}

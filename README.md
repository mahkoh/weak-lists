# weak-lists

[![crates.io](https://img.shields.io/crates/v/weak-lists.svg)](http://crates.io/crates/weak-lists)
[![docs.rs](https://docs.rs/weak-lists/badge.svg)](http://docs.rs/weak-lists)

This crate provides list types that hold weak references to their elements. These
lists allow concurrent iteration over and modification of the lists with reasonable
outcomes.

## Example

Consider a service that allows clients to register callbacks:

```rust
use std::sync::Arc;
use {
    weak_lists::{SyncWeakList, SyncWeakListElement},
    std::{
        array,
    },
};

pub struct Service {
    callbacks: SyncWeakList<dyn Callback>,
}

pub trait Callback {
    fn run(&self);
}

impl Service {
    pub fn register_callback(&self, callback: &SyncWeakListElement<dyn Callback>) {
        callback.attach(&self.callbacks);
    }

    pub fn run_callbacks(&self) {
        for callback in &self.callbacks {
            callback.run();
        }
    }
}

struct Client {
    id: usize,
    entry: SyncWeakListElement<dyn Callback>,
}

impl Callback for Client {
    fn run(&self) {
        eprintln!("Callback {} invoked", self.id);
        if self.id == 1 {
            self.entry.detach();
        }
    }
}

fn main() {
    let service = Service {
        callbacks: Default::default(),
    };
    let clients = array::from_fn::<_, 3, _>(|id| {
        Arc::<Client>::new_cyclic(|slf| Client {
            id,
            entry: SyncWeakListElement::new(slf.clone()),
        })
    });
    for client in &clients {
        service.register_callback(&client.entry);
    }
    service.run_callbacks();
    // Callback 0 invoked
    // Callback 1 invoked
    // Callback 2 invoked
    service.run_callbacks();
    // Callback 0 invoked
    // Callback 2 invoked
}
```

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.

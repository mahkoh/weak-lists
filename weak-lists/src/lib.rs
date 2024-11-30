//! This crate provides list types that hold weak references to their elements. These
//! lists allow concurrent iteration over and modification of the lists with reasonable
//! outcomes.

#![no_std]
extern crate alloc;

#[cfg(feature = "sync")]
pub mod sync;
pub mod unsync;

#[cfg(feature = "sync")]
pub use sync::{SyncWeakList, SyncWeakListElement};
pub use unsync::{WeakList, WeakListElement};

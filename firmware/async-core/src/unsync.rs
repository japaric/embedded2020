//! `!Sync` task synchronization primitives

mod mutex;
pub mod spsc;

pub use mutex::Mutex;

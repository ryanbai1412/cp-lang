//! The one place a tool waits: a child's exit (`on_exit`), a file change
//! (`files`) and a rate limit (`Pace`). Each blocks instead of polling where
//! the platform allows it; debt-ratchet's `poll_sleeps` fails a sleep
//! anywhere else in shipped Rust.

mod child;
pub mod files;
mod pace;

pub use child::{kill, on_exit, Exit};
pub use pace::Pace;

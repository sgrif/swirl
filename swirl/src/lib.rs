#![deny(warnings)]

#[macro_use]
extern crate diesel;

#[doc(hidden)]
pub extern crate inventory;

mod job;
mod registry;
mod runner;
mod storage;

pub mod errors;
pub mod db;
pub mod schema;

pub use swirl_proc_macro::*;

pub use errors::*;
pub use job::*;
pub use registry::Registry;
pub use runner::*;

#[doc(hidden)]
pub use registry::JobVTable;

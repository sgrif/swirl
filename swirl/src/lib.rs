#![deny(warnings)]

#[macro_use]
extern crate diesel;

#[doc(hidden)]
pub extern crate inventory;

mod db;
mod job;
mod registry;
mod runner;
mod storage;

pub mod errors;
pub mod schema;

pub use db::DieselPool;
pub use errors::*;
pub use job::*;
pub use registry::Registry;
pub use runner::*;

#[doc(hidden)]
pub use registry::JobVTable;

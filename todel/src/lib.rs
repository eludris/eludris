//! A simple crate that houses most of the Eludris models & shared logic.

#[cfg(feature = "logic")]
#[macro_use]
extern crate lazy_static;
#[macro_use]
pub extern crate todel_codegen;

pub mod bitflag;
pub mod conf;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "logic")]
pub mod ids;
pub mod models;

pub use conf::Conf;

#[cfg(feature = "logic")]
pub use todel_codegen::*;

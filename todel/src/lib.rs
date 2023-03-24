//! A simple crate that houses most of the Eludris models & shared logic.

#[cfg(feature = "logic")]
#[macro_use]
extern crate lazy_static;

pub mod conf;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "logic")]
pub mod ids;
pub mod models;

pub use conf::Conf;

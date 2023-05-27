//! A collection of models and some related function implementations for eludris.

mod files;
mod gateway;
mod info;
mod messages;
mod response;
mod users;

pub use files::*;
pub use gateway::*;
pub use info::*;
pub use messages::*;
pub use response::*;
pub use users::*;

#[cfg(feature = "logic")]
mod logic;

#[cfg(feature = "logic")]
pub use logic::*;

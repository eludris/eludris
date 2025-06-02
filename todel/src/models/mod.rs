//! A collection of models and some related function implementations for eludris.

mod categories;
mod channels;
mod embeds;
mod emojis;
mod files;
mod gateway;
mod info;
mod members;
mod messages;
mod response;
mod sessions;
mod spheres;
mod users;

pub use categories::*;
pub use channels::*;
pub use embeds::*;
pub use emojis::*;
pub use files::*;
pub use gateway::*;
pub use info::*;
pub use members::*;
pub use messages::*;
pub use response::*;
pub use sessions::*;
pub use spheres::*;
pub use users::*;

#[cfg(feature = "logic")]
mod logic;

#[cfg(feature = "logic")]
pub use logic::*;

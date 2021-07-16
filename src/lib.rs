pub mod logger;
pub mod meta;
pub mod server;
mod tests;
pub mod client;

pub const DEFAULT_KEY_TYPE: &str = "ed25519";

#[macro_use]
extern crate log;
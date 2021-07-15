mod logger;
mod meta;
mod server;
mod tests;

pub const DEFAULT_KEY_TYPE: &str = "ed25519";

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
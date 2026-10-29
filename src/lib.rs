#![forbid(unsafe_code)]

pub mod browser;
pub mod cli;
pub mod config;
pub mod control;
pub mod http;
pub mod https;
pub mod mcp;
pub mod proxy;
pub mod reload;
pub mod security;
pub mod server;
pub mod spa;
pub mod watcher;
pub mod websocket;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

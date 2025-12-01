//! orders-hex: hexagonal Orders API library (core + inbound HTTP)

pub mod config;
pub mod errors;

pub mod application;

pub use orders_types::{domain, ports};

pub mod inbound; // HTTP adapter (server + handlers)

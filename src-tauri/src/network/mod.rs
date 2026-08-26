/// Network module for Lumin — handles peer discovery and local-network services.
pub mod auth;
pub mod dns_sd;
pub mod reliable;
pub mod server;

#[allow(unused_imports)]
pub use reliable::{EventDeduplicator, PendingQueue};

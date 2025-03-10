pub mod consumer_groups;
pub mod consumer_offsets;
pub mod diagnostics;
pub mod error;
pub mod http_server;
pub mod jwt;
mod mapper;
pub mod messages;
pub mod metrics;
pub mod partitions;
pub mod personal_access_tokens;
mod shared;
pub mod streams;
pub mod system;
pub mod topics;
pub mod users;

pub const COMPONENT: &str = "HTTP";

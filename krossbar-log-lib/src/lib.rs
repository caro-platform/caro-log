pub mod logger;
pub mod logger_client;

pub type Error = Box<dyn std::error::Error + Sync + Send>;
pub type Result<T> = std::result::Result<T, Error>;

pub use logger::Logger;

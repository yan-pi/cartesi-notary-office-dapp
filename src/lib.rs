pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use domain::{Document, NotarizationReceipt};
pub use infrastructure::database::{DocumentRepository, SqliteRepository};

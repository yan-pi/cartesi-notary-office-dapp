pub mod application;
pub mod domain;
pub mod handlers;
pub mod infrastructure;

// Re-export commonly used types
pub use application::{NotarizeUseCase, VerificationResult, VerifyUseCase};
pub use domain::{Document, NotarizationReceipt};
pub use infrastructure::database::{DocumentRepository, SqliteRepository};

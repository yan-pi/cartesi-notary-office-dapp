use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content_hash: String,
    pub file_name: String,
    pub mime_type: String,
    pub submitted_by: String,
    pub created_at: i64,
}

impl Document {
    pub fn new(content: &[u8], file_name: &str, mime_type: &str, submitted_by: &str) -> Self {
        // Generate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash_bytes = hasher.finalize();
        let content_hash = format!("{:x}", hash_bytes);

        // Generate unique ID
        let id = uuid::Uuid::new_v4().to_string();

        // Get current timestamp
        let created_at = chrono::Utc::now().timestamp();

        Self {
            id,
            content_hash,
            file_name: file_name.to_string(),
            mime_type: mime_type.to_string(),
            submitted_by: submitted_by.to_string(),
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_output_length() {
        let content = b"test";
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());
        assert_eq!(hash.len(), 64);
    }
}

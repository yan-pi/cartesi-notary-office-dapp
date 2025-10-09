use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotarizationReceipt {
    pub document_id: String,
    pub content_hash: String,
    pub notarized_at: i64,
    pub block_number: u64,
    pub proof: String,
}

impl NotarizationReceipt {
    pub fn new(
        document_id: String,
        content_hash: String,
        notarized_at: i64,
        block_number: u64,
    ) -> Self {
        let proof = format!("sha256:{}@{}", content_hash, notarized_at);

        Self {
            document_id,
            content_hash,
            notarized_at,
            block_number,
            proof,
        }
    }
}

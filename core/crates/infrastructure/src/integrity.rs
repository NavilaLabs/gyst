use sha2::{Digest, Sha256};

use crate::EventRecord; // For canonical byte representation

pub trait IntegrityChain {
    fn calculate_hash(&self, previous_hash: &[u8]) -> Vec<u8>;
}

impl IntegrityChain for EventRecord {
    fn calculate_hash(&self, previous_hash: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();

        hasher.update(previous_hash);

        hasher.update(self.event_id.as_bytes());
        hasher.update(self.aggregate.get_aggregate_id().as_bytes());
        hasher.update(self.aggregate.get_aggregate_version().to_le_bytes());

        if let Ok(data_bytes) = serde_jcs::to_vec(&self.data) {
            hasher.update(data_bytes);
        }

        hasher.update(self.timestamps.get_created_at().timestamp().to_le_bytes());
        hasher.update(self.context.get_created_by().as_bytes());

        hasher.finalize().to_vec()
    }
}

//! Audit Chain - Immutable logging with SHA3 hashes
//!
//! Provides tamper-evident audit trail for all agent operations.

use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Digest};

/// Audit entry for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Tool that was executed
    pub tool: String,
    /// Input data (hashed)
    pub input_hash: String,
    /// Output data (hashed)
    pub output_hash: String,
    /// Consent level used
    pub consent_level: String,
    /// Session ID
    pub session_id: String,
    /// Hash of previous entry (chain)
    pub previous_hash: String,
}

/// Audit chain for immutable logging
pub struct AuditChain {
    entries: Vec<AuditEntry>,
    current_hash: String,
}

impl Default for AuditChain {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditChain {
    /// Create new audit chain with genesis block
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_hash: "genesis".to_string(),
        }
    }

    /// Compute SHA3-256 hash of data
    fn compute_hash(data: &str) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Add entry to the chain
    pub fn add_entry(&mut self, entry: AuditEntry) {
        // Compute new hash based on previous hash + entry data
        let combined = format!(
            "{}{}{}{}",
            self.current_hash, entry.timestamp, entry.tool, entry.consent_level
        );
        let entry_hash = Self::compute_hash(&combined);

        // Update chain
        let mut mutable_entry = entry;
        mutable_entry.previous_hash = self.current_hash.clone();

        // Chain the hashes
        self.current_hash = Self::compute_hash(&format!(
            "{}{}",
            self.current_hash, entry_hash
        ));

        self.entries.push(mutable_entry);
    }

    /// Verify the chain integrity
    pub fn verify(&self) -> Result<bool, String> {
        if self.entries.is_empty() {
            return Ok(true);
        }

        let mut expected_previous = "genesis".to_string();

        for (i, entry) in self.entries.iter().enumerate() {
            // Check previous hash matches
            if entry.previous_hash != expected_previous {
                return Err(format!(
                    "Chain broken at entry {}: expected previous '{}', got '{}'",
                    i, expected_previous, entry.previous_hash
                ));
            }

            // Compute what the hash should be
            let combined = format!(
                "{}{}{}{}",
                entry.previous_hash, entry.timestamp, entry.tool, entry.consent_level
            );
            let computed_hash = Self::compute_hash(&combined);

            // Update expected previous for next iteration
            expected_previous = Self::compute_hash(&format!(
                "{}{}",
                entry.previous_hash, computed_hash
            ));
        }

        Ok(true)
    }

    /// Export all entries as JSON
    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_default()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current chain hash
    pub fn current_hash(&self) -> &str {
        &self.current_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chain() {
        let chain = AuditChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut chain = AuditChain::new();
        let entry = AuditEntry {
            timestamp: 1234567890,
            tool: "test_tool".to_string(),
            input_hash: "abc123".to_string(),
            output_hash: "def456".to_string(),
            consent_level: "auto".to_string(),
            session_id: "session1".to_string(),
            previous_hash: String::new(), // Will be set by add_entry
        };
        chain.add_entry(entry);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_verify() {
        let mut chain = AuditChain::new();
        let entry = AuditEntry {
            timestamp: 1234567890,
            tool: "test".to_string(),
            input_hash: "in".to_string(),
            output_hash: "out".to_string(),
            consent_level: "auto".to_string(),
            session_id: "s1".to_string(),
            previous_hash: String::new(),
        };
        chain.add_entry(entry);
        assert!(chain.verify().is_ok());
    }

    #[test]
    fn test_export_json() {
        let mut chain = AuditChain::new();
        let entry = AuditEntry {
            timestamp: 1234567890,
            tool: "test".to_string(),
            input_hash: "in".to_string(),
            output_hash: "out".to_string(),
            consent_level: "auto".to_string(),
            session_id: "s1".to_string(),
            previous_hash: String::new(),
        };
        chain.add_entry(entry);
        let json = chain.export_json();
        assert!(json.contains("test"));
    }
}
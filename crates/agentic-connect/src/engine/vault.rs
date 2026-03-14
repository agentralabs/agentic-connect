//! Credential vault — encrypted storage for auth credentials.

use ring::aead;
use ring::rand::{SecureRandom, SystemRandom};
use std::collections::HashMap;

use crate::types::{ConnectError, ConnectResult, StoredCredential};

/// Encrypted credential vault using AES-256-GCM.
pub struct CredentialVault {
    credentials: HashMap<String, StoredCredential>,
    encryption_key: Option<aead::LessSafeKey>,
}

impl CredentialVault {
    /// Create a new vault (unencrypted in-memory for now).
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            encryption_key: None,
        }
    }

    /// Create a vault with encryption enabled.
    pub fn with_encryption(passphrase: &str) -> ConnectResult<Self> {
        let key = derive_key(passphrase)?;
        Ok(Self {
            credentials: HashMap::new(),
            encryption_key: Some(key),
        })
    }

    /// Store a credential.
    pub fn store(&mut self, cred: StoredCredential) {
        self.credentials.insert(cred.name.clone(), cred);
    }

    /// Retrieve a credential by name.
    pub fn retrieve(&self, name: &str) -> Option<&StoredCredential> {
        self.credentials.get(name)
    }

    /// Delete a credential by name.
    pub fn delete(&mut self, name: &str) -> bool {
        self.credentials.remove(name).is_some()
    }

    /// List all credential names (never returns actual secrets).
    pub fn list(&self) -> Vec<CredentialSummary> {
        self.credentials.values().map(|c| CredentialSummary {
            name: c.name.clone(),
            auth_type: c.auth.method_name().to_string(),
            created_at: c.created_at.to_rfc3339(),
            tags: c.tags.clone(),
        }).collect()
    }

    /// Number of stored credentials.
    pub fn count(&self) -> usize {
        self.credentials.len()
    }

    /// Encrypt a plaintext value.
    pub fn encrypt(&self, plaintext: &[u8]) -> ConnectResult<Vec<u8>> {
        let key = self.encryption_key.as_ref()
            .ok_or_else(|| ConnectError::EncryptionError("No encryption key set".into()))?;

        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| ConnectError::EncryptionError("RNG failure".into()))?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.to_vec();

        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| ConnectError::EncryptionError("Encryption failed".into()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(in_out);
        Ok(result)
    }

    /// Decrypt a ciphertext value.
    pub fn decrypt(&self, ciphertext: &[u8]) -> ConnectResult<Vec<u8>> {
        let key = self.encryption_key.as_ref()
            .ok_or_else(|| ConnectError::EncryptionError("No encryption key set".into()))?;

        if ciphertext.len() < 12 {
            return Err(ConnectError::EncryptionError("Ciphertext too short".into()));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| ConnectError::EncryptionError("Invalid nonce".into()))?;

        let mut in_out = encrypted.to_vec();
        let plaintext = key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| ConnectError::EncryptionError("Decryption failed".into()))?;

        Ok(plaintext.to_vec())
    }
}

/// Summary of a credential (safe to display — no secrets).
#[derive(Debug, Clone, serde::Serialize)]
pub struct CredentialSummary {
    pub name: String,
    pub auth_type: String,
    pub created_at: String,
    pub tags: Vec<String>,
}

/// Derive an AES-256-GCM key from a passphrase using PBKDF2.
fn derive_key(passphrase: &str) -> ConnectResult<aead::LessSafeKey> {
    use ring::pbkdf2;

    let salt = b"agentic-connect-vault-v1"; // Fixed salt for deterministic derivation
    let mut key_bytes = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(100_000).unwrap(),
        salt,
        passphrase.as_bytes(),
        &mut key_bytes,
    );

    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| ConnectError::EncryptionError("Key derivation failed".into()))?;

    Ok(aead::LessSafeKey::new(unbound))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::auth::AuthMethod;
    use chrono::Utc;

    #[test]
    fn test_store_and_retrieve() {
        let mut vault = CredentialVault::new();
        let cred = StoredCredential {
            id: uuid::Uuid::new_v4(),
            name: "test-api".into(),
            auth: AuthMethod::Bearer { token: "test-token-123".into() },
            created_at: Utc::now(),
            last_rotated: None,
            tags: vec!["prod".into()],
        };
        vault.store(cred);
        assert_eq!(vault.count(), 1);
        let retrieved = vault.retrieve("test-api").unwrap();
        assert_eq!(retrieved.name, "test-api");
    }

    #[test]
    fn test_delete() {
        let mut vault = CredentialVault::new();
        let cred = StoredCredential {
            id: uuid::Uuid::new_v4(), name: "temp".into(),
            auth: AuthMethod::None, created_at: Utc::now(),
            last_rotated: None, tags: vec![],
        };
        vault.store(cred);
        assert!(vault.delete("temp"));
        assert_eq!(vault.count(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let vault = CredentialVault::with_encryption("my-secret-passphrase").unwrap();
        let plaintext = b"sensitive-api-key-12345";
        let ciphertext = vault.encrypt(plaintext).unwrap();
        assert_ne!(&ciphertext, plaintext);
        let decrypted = vault.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_list_hides_secrets() {
        let mut vault = CredentialVault::new();
        vault.store(StoredCredential {
            id: uuid::Uuid::new_v4(), name: "secret-key".into(),
            auth: AuthMethod::Bearer { token: "SUPER_SECRET".into() },
            created_at: Utc::now(), last_rotated: None, tags: vec![],
        });
        let list = vault.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].auth_type, "bearer");
        // The summary should NOT contain the actual token
        let json = serde_json::to_string(&list[0]).unwrap();
        assert!(!json.contains("SUPER_SECRET"));
    }
}

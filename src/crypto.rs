#![allow(dead_code)]

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{PasswordHash, SaltString, rand_core::OsRng}};
use blake3::Hasher;
use crate::error::TywindbError;

pub struct Crypto {
    key: [u8; 32],
}

impl Crypto {
    pub fn from_password(password: &str) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(password.as_bytes());
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash.as_bytes());
        Self { key }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, TywindbError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| TywindbError::Encryption(e.to_string()))?;
        let nonce = Nonce::from_slice(&self.key[..12]);
        cipher.encrypt(nonce, data)
            .map_err(|e| TywindbError::Encryption(e.to_string()))
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, TywindbError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| TywindbError::Encryption(e.to_string()))?;
        let nonce = Nonce::from_slice(&self.key[..12]);
        cipher.decrypt(nonce, data)
            .map_err(|e| TywindbError::Encryption(e.to_string()))
    }

    pub fn hash_password(password: &str) -> Result<String, TywindbError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| TywindbError::Encryption(e.to_string()))?;
        Ok(hash.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, TywindbError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| TywindbError::Encryption(e.to_string()))?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

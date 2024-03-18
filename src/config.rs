use std::{collections::HashMap, path::Path};

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, KeyInit, XChaCha20Poly1305,
};
use serde::{Deserialize, Serialize};

use crate::store::{Entry, PasswordStore};
use crate::Result;

fn key_from_password(password: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let mut password_bytes = password.bytes();

    for i in 0..32 {
        key[i] = match password_bytes.nth(i) {
            Some(val) => val,
            None => 0,
        }
    }

    key
}

impl From<HashMap<String, (String, String)>> for Config {
    fn from(value: HashMap<String, (String, String)>) -> Self {
        Config {
            store: PasswordStore::from(value),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    store: PasswordStore,
}

impl Config {
    pub fn new() -> Self {
        Config {
            store: PasswordStore::new(),
        }
    }

    pub fn add_entry(&mut self, account: &str, entry: Entry) -> Result<()> {
        self.store.add_entry(account, entry)
    }

    pub fn get_entry(&self, account: &str) -> Option<&Entry> {
        self.store.get_entry(account)
    }

    pub fn get_all_entries(&self) -> impl Iterator<Item = (&String, &Entry)> {
        self.store.entries()
    }

    /// Reads and decrypts config from file.
    pub fn from_bytes(contents: Vec<u8>, db_password: &str) -> Result<Self> {
        let nonce = &contents[0..24];
        let payload = &contents[24..];

        let cipher = XChaCha20Poly1305::new(&key_from_password(db_password).into());

        let original = cipher
            .decrypt(nonce.into(), payload)
            .map_err(|_| anyhow!("Failed to decrypt config file."))?;

        Ok(serde_yaml::from_slice(&original)?)
    }

    /// Get encrypted bytes.
    pub fn encrypt_bytes(&self, db_password: &str) -> Result<Vec<u8>> {
        let yaml = serde_yaml::to_string(self)?;

        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let cipher = XChaCha20Poly1305::new(&key_from_password(db_password).into());

        let ciphertext = cipher
            .encrypt(&nonce, yaml.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt config."))?;

        Ok(vec![nonce.to_vec(), ciphertext].concat())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    const PASSWORD: &str = "TEST_PASS";

    fn mock_config() -> Config {
        Config::from(HashMap::from([
            (
                "foo".to_string(),
                ("foo@gmail.com".to_string(), "pass1".to_string()),
            ),
            (
                "bar".to_string(),
                ("bar@gmail.com".to_string(), "pass2".to_string()),
            ),
        ]))
    }

    #[test]
    fn correct_password() {
        let config = Config::new();
        let config_bytes = config.encrypt_bytes(PASSWORD).unwrap();
        let res = Config::from_bytes(config_bytes, PASSWORD);
        assert!(res.is_ok());
    }

    #[test]
    fn wrong_password() {
        let config = Config::new();
        let config_bytes = config.encrypt_bytes(PASSWORD).unwrap();
        let res = Config::from_bytes(config_bytes, "WRONG_PASSWORD");
        assert!(res.is_err());
    }

    #[test]
    fn add_entry() {
        let mut config = Config::new();
        config
            .add_entry(
                "example.com",
                Entry {
                    username: "foo".to_string(),
                    password: "baz".to_string(),
                },
            )
            .unwrap();

        let config_bytes = config.encrypt_bytes(PASSWORD).unwrap();

        let config_from_bytes = Config::from_bytes(config_bytes, PASSWORD).unwrap();

        let entry = config_from_bytes.get_entry("example.com").unwrap();

        assert_eq!(entry.username, "foo");
        assert_eq!(entry.password, "baz");
    }
}

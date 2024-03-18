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
        //self.store.get_password(account)
        self.store.get_entry(account)
    }

    pub fn get_all_entries(&self) -> impl Iterator<Item = (&String, &Entry)> {
        self.store.entries()
    }

    /// Reads and decrypts config from file.
    pub fn from_file(path: impl AsRef<Path>, db_password: &str) -> Result<Self> {
        let contents = std::fs::read(path)?;

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
    use std::collections::HashMap;

    use tempdir::TempDir;

    use super::*;

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
    /// We can create an empty db, write it to disk and decrypt it. The password store should be
    /// empty.
    fn empty_db() {
        let dir = TempDir::new("empty_db").unwrap();
        let db_name = "my_test_db";

        create_empty_db(db_name, PASSWORD, dir.path()).unwrap();

        let config: Config =
            Config::from_file(dir.path().join("my_test_db.pwm"), PASSWORD).unwrap();

        assert!(config.store.is_empty());
    }

    #[test]
    /// We can write db to disk, add password to db on disk and read the password from the store.
    fn store_password() {
        let config = mock_config();
        let dir = TempDir::new("store_password").unwrap();
        let file_path = dir.path().join("test.pwm");

        create_db("test", PASSWORD, config, dir.path()).unwrap();
        add_entry(
            &file_path,
            PASSWORD,
            "baz",
            Entry {
                username: "example.com".to_string(),
                password: "pass3".to_string(),
            },
        )
        .unwrap();
        let entry = get_entry(&file_path, PASSWORD, "baz").unwrap();

        assert_eq!(entry.password, String::from("pass3"));
    }
}

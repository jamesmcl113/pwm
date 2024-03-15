use std::{collections::HashMap, path::Path};

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, KeyInit, XChaCha20Poly1305,
};
use serde::{Deserialize, Serialize};

use crate::store::PasswordStore;
use crate::Result;

const PASSWORD: &str = "TEST_PASS";

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

impl From<HashMap<String, String>> for Config {
    fn from(value: HashMap<String, String>) -> Self {
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

    pub fn add_entry(&mut self, account: &str, password: &str) -> Result<()> {
        self.store
            .add_password(account.to_string(), password.to_string())
    }

    pub fn get_entry(&self, account: &str) -> Option<&String> {
        self.store.get_password(account)
    }

    /// Reads and decrypts config from file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read(path)?;

        let nonce = &contents[0..24];
        let payload = &contents[24..];

        let cipher = XChaCha20Poly1305::new(&key_from_password(PASSWORD).into());

        let original = cipher
            .decrypt(nonce.into(), payload)
            .map_err(|_| anyhow!("Failed to decrypt config file."))?;

        Ok(serde_yaml::from_slice(&original)?)
    }

    /// Get encrypted bytes.
    pub fn encrypt_bytes(&self) -> Result<Vec<u8>> {
        let yaml = serde_yaml::to_string(self)?;

        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let cipher = XChaCha20Poly1305::new(&key_from_password(PASSWORD).into());

        let ciphertext = cipher
            .encrypt(&nonce, yaml.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt config."))?;

        Ok(vec![nonce.to_vec(), ciphertext].concat())
    }
}

/// Creates an empty db file at `path/{name}.pwm`
pub fn create_empty_db(name: &str, path: impl AsRef<Path>) -> Result<()> {
    let config = Config::new();

    std::fs::write(
        path.as_ref().join(format!("{name}.pwm")),
        config.encrypt_bytes()?,
    )?;
    Ok(())
}

pub fn create_db(name: &str, config: Config, path: impl AsRef<Path>) -> Result<()> {
    std::fs::write(
        path.as_ref().join(format!("{name}.pwm")),
        config.encrypt_bytes()?,
    )?;

    Ok(())
}

pub fn add_entry(
    db_path: impl AsRef<Path>,
    account: &str,
    password: &str,
) -> Result<()> {
    let mut config: Config = Config::from_file(&db_path)?;

    config.add_entry(account, password)?;

    std::fs::write(db_path, config.encrypt_bytes()?)?;

    Ok(())
}

pub fn get_entry(db_path: impl AsRef<Path>, account: &str) -> Result<String> {
    let config = Config::from_file(&db_path)?;

    config
        .get_entry(account)
        .map(|p| p.clone())
        .ok_or(anyhow!("Account `{account}` not found in db."))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use tempdir::TempDir;

    use super::*;

    fn mock_config() -> Config {
        Config::from(HashMap::from([
            ("foo".to_string(), "pass1".to_string()),
            ("bar".to_string(), "pass2".to_string()),
        ]))
    }

    #[test]
    /// We can create an empty db, write it to disk and decrypt it. The password store should be
    /// empty.
    fn empty_db() {
        let dir = TempDir::new("empty_db").unwrap();
        let db_name = "my_test_db";

        create_empty_db(db_name, dir.path()).unwrap();

        let config: Config =
            Config::from_file(dir.path().join("my_test_db.pwm")).unwrap();

        assert!(config.store.is_empty());
    }

    #[test]
    /// We can write db to disk, add password to db on disk and read the password from the store.
    fn store_password() {
        let config = mock_config();
        let dir = TempDir::new("store_password").unwrap();
        let file_path = dir.path().join("test.pwm");

        create_db("test", config, dir.path()).unwrap();
        add_entry(&file_path, "baz", "pass3").unwrap();

        let config = Config::from_file(&file_path).unwrap();

        assert_eq!(config.get_entry("baz"), Some(&"pass3".to_owned()));
    }
}

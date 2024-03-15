use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Result;

impl From<HashMap<String, String>> for PasswordStore {
    fn from(value: HashMap<String, String>) -> Self {
        PasswordStore { data: value }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasswordStore {
    data: HashMap<String, String>,
}

impl PasswordStore {
    pub fn new() -> PasswordStore {
        PasswordStore {
            data: HashMap::new(),
        }
    }

    pub fn get_password(&self, account: &str) -> Option<&String> {
        self.data.get(account)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn add_password(&mut self, account: String, password: String) -> Result<()> {
        match self.data.get(&account) {
            Some(_) => Err(anyhow::anyhow!("Account already added to store!")),
            None => {
                self.data.insert(account, password);
                Ok(())
            }
        }
    }
}

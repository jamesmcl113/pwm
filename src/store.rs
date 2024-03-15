use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Result;

type AccountName = String;

/// An entry in the password store.
#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    /// Email or username associated with this entry.
    pub username: String,

    /// Password associated with this entry.
    pub password: String,
}

impl From<HashMap<AccountName, Entry>> for PasswordStore {
    fn from(value: HashMap<AccountName, Entry>) -> Self {
        PasswordStore { data: value }
    }
}

impl From<HashMap<AccountName, (String, String)>> for PasswordStore {
    fn from(value: HashMap<AccountName, (String, String)>) -> Self {
        PasswordStore {
            data: value
                .iter()
                .map(|(acc, (username, pass))| {
                    (
                        acc.to_owned(),
                        Entry {
                            username: username.to_string(),
                            password: pass.to_owned(),
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasswordStore {
    data: HashMap<AccountName, Entry>,
}

impl PasswordStore {
    pub fn new() -> PasswordStore {
        PasswordStore {
            data: HashMap::new(),
        }
    }

    pub fn get_password(&self, account: &str) -> Option<&String> {
        self.data.get(account).map(|entry| &entry.password)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn add_password(
        &mut self,
        account: String,
        username: String,
        password: String,
    ) -> Result<()> {
        match self.data.get(&account) {
            Some(_) => Err(anyhow::anyhow!("Account already added to store!")),
            None => {
                self.data.insert(account, Entry { username, password });
                Ok(())
            }
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &Entry)> {
        self.data.iter()
    }

    pub fn add_entry(&mut self, account: &str, entry: Entry) -> Result<()> {
        match self.data.get(account) {
            Some(_) => Err(anyhow::anyhow!("Account already added to store!")),
            None => {
                self.data.insert(account.to_string(), entry);
                Ok(())
            }
        }
    }

    pub fn get_entry(&self, account: &str) -> Option<&Entry> {
        self.data.get(account)
    }
}

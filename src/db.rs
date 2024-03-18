use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::{
    config::Config,
    store::{self},
    Result,
};

pub struct DB {
    password: String,
}

impl DB {
    pub fn new(password: &str) -> Self {
        DB {
            password: password.to_owned(),
        }
    }

    fn check_file_path(db_file: impl AsRef<Path>) -> Result<()> {
        match db_file.as_ref().extension() {
            Some(ext) => {
                if ext != "pwm" {
                    return Err(anyhow!("DB file must be a .pwm file."));
                }
            }
            None => return Err(anyhow!("DB file must end with .pwm")),
        }

        Ok(())
    }

    pub fn create_empty(&self, file_path: impl AsRef<Path>) -> Result<()> {
        DB::check_file_path(&file_path)?;

        if file_path.as_ref().exists() {
            return Err(anyhow!(
                "Could not create DB. {:?} already exists.",
                &file_path.as_ref()
            ));
        }

        let config = Config::new();
        std::fs::write(&file_path, config.encrypt_bytes(&self.password)?)?;

        println!("DB created successfully at `{:?}`.", file_path.as_ref());

        Ok(())
    }

    pub fn add_entry(
        &self,
        file_path: impl AsRef<Path>,
        account: &str,
        entry: store::Entry,
    ) -> Result<()> {
        DB::check_file_path(&file_path)?;

        let mut config: Config = Config::from_file(&file_path, &self.password)?;

        config.add_entry(account, entry)?;

        std::fs::write(&file_path, config.encrypt_bytes(&self.password)?)?;
        Ok(())
    }

    pub fn get_entry(
        &self,
        file_path: impl AsRef<Path>,
        account: &str,
    ) -> Result<store::Entry> {
        DB::check_file_path(&file_path)?;

        let config = Config::from_file(&file_path, &self.password)?;

        config
            .get_entry(account)
            .map(|entry| entry.clone())
            .ok_or(anyhow!("No entry found for account `{account}`"))
    }
    pub fn get_all_entries(
        &self,
        file_path: impl AsRef<Path>,
    ) -> Result<Vec<(String, store::Entry)>> {
        DB::check_file_path(&file_path)?;

        let config = Config::from_file(&file_path, &self.password)?;

        Ok(config
            .get_all_entries()
            .map(|(acc, entry)| (acc.clone(), entry.clone()))
            .collect())
    }
}

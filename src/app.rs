use std::path::Path;

use crate::cli::{Cli, Command, JsonPayload};
use crate::store::Entry;
use crate::{config, Result};

use anyhow::anyhow;

pub struct App {
    cli: Cli,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        App { cli }
    }

    pub fn run(self) -> Result<()> {
        let cli = self.cli;

        let db_password = match &cli.password {
            Some(pass) => pass.clone(),
            None => {
                let ev_password = std::env::var("PWM_PASS");
                if let Ok(pass) = ev_password {
                    pass
                } else {
                    return Err(anyhow!("Password not supplied and PWM_PASS not found in environment. Quitting..."));
                }
            }
        };

        match cli.command {
            Some(cmd) => match cmd {
                Command::Add {
                    db,
                    account,
                    username,
                    pass,
                } => execute_add_command(&db, &db_password, account, username, pass),
                Command::Get { db, account } => {
                    execute_get_command(&db, &db_password, account)
                }
                Command::Init { name } => {
                    match config::create_empty_db(
                        &name,
                        &db_password,
                        std::env::current_dir()?,
                    ) {
                        Ok(_) => {
                            println!("Successfully created db `{name}`");
                            Ok(())
                        }
                        Err(e) => {
                            Err(anyhow!("Failed to run command `init`. Err = {e:?}"))
                        }
                    }
                }
                Command::AddJson { db, payload } => {
                    let payload: JsonPayload = serde_json::from_str(&payload)?;
                    let path = std::env::current_dir()?.join(&db);

                    execute_add_command(
                        &path,
                        &db_password,
                        payload.account,
                        payload.username,
                        payload.password,
                    )
                }
            },
            None => Ok(()),
        }
    }
}

fn execute_get_command(
    db_path: impl AsRef<Path>,
    db_password: &str,
    account: Option<String>,
) -> Result<()> {
    let path = std::env::current_dir()?.join(&db_path);

    if let Some(account) = account {
        match config::get_entry(&path, &db_password, &account) {
            Err(e) => Err(anyhow!("Failed to run command `get`. Err = {:?}", e)),
            Ok(Entry { username, password }) => {
                println!("`{account}`:\n  {username}: {password}",);
                Ok(())
            }
        }
    } else {
        match config::get_all_entries(&path, &db_password) {
            Ok(entries) => {
                for (account, Entry { username, password }) in entries {
                    println!("`{account}`:\n  {username}: {password}",)
                }
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to run command `get`. Err = {:?}", e)),
        }
    }
}

fn execute_add_command(
    db_path: impl AsRef<Path>,
    db_password: &str,
    account: String,
    username: String,
    password: String,
) -> Result<()> {
    let path = std::env::current_dir()?.join(&db_path);

    match config::add_entry(
        &path,
        db_password,
        &account,
        Entry { username, password },
    ) {
        Ok(_) => {
            println!("Added account: {account} to {:?}", db_path.as_ref());
            Ok(())
        }
        Err(e) => Err(anyhow!("Failed to run command `add`. Err = {:?}", e)),
    }
}

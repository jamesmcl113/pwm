use std::path::Path;

use crate::cli::{Cli, Command, JsonPayload};
use crate::db::DB;
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

        let db = DB::new(&db_password);

        match cli.command {
            Some(cmd) => match cmd {
                Command::Add {
                    db_file,
                    account,
                    username,
                    pass,
                } => {
                    let entry = Entry {
                        username,
                        password: pass,
                    };

                    db.add_entry(&db_file, &account, entry)
                }
                Command::Get { db_file, account } => {
                    execute_get_command(&db, &db_file, account)
                }
                Command::Init { name } => db.create_empty(
                    std::env::current_dir()?.join(format!("{name}.pwm")),
                ),
                Command::AddJson { db_file, payload } => {
                    let payload: JsonPayload = serde_json::from_str(&payload)?;
                    let path = std::env::current_dir()?.join(&db_file);

                    db.add_entry(
                        path,
                        &payload.account,
                        Entry {
                            username: payload.username,
                            password: payload.password,
                        },
                    )
                }
            },
            None => Ok(()),
        }
    }
}

fn display_entry(account: &str, entry: &Entry) {
    println!("`{account}`:\n  {}: {}", entry.username, entry.password);
}

fn execute_get_command(
    db: &DB,
    db_path: impl AsRef<Path>,
    account: Option<String>,
) -> Result<()> {
    let path = std::env::current_dir()?.join(&db_path);

    if let Some(account) = account {
        let entry = db.get_entry(&path, &account)?;
        display_entry(&account, &entry);
    } else {
        let entries = db.get_all_entries(&path)?;
        entries.iter().for_each(|(account, entry)| {
            display_entry(&account, &entry);
        })
    }

    Ok(())
}

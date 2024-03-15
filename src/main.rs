mod config;
mod store;

use std::path::Path;

use store::Entry;

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub type Result<T> = anyhow::Result<T>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// If not supplied, pwm will use $PWM_PASS instead.
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Add password to file.
    Add {
        /// The password file
        db: String,

        /// Name for the account entry e.g. 'github.com'
        #[arg(short)]
        account: String,

        /// Password for the account
        #[arg(short)]
        pass: String,

        /// Username for the account
        #[arg(short)]
        username: String,
    },
    /// Add password to file from JSON payload. Payload *MUST* contain account, username and
    /// password keys.
    AddJson {
        db: String,

        #[arg(short)]
        payload: String,
    },
    /// Print password from file. If `account` is not provided, prints all entries in db.
    Get { db: String, account: Option<String> },
    /// Create an empty pwm file.
    Init { name: String },
}

#[derive(Serialize, Deserialize)]
struct JsonPayload {
    account: String,
    username: String,
    password: String,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

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
            } => {
                execute_add_command(&db, &db_password, account, username, pass)?;
            }
            Command::Get { db, account } => {
                let path = std::env::current_dir()?.join(&db);

                if let Some(account) = account {
                    match config::get_entry(&path, &db_password, &account) {
                        Err(e) => {
                            return Err(anyhow!(
                                "Failed to run command `get`. Err = {:?}",
                                e
                            ))
                        }
                        Ok(Entry { username, password }) => {
                            println!("`{account}`:\n  {username}: {password}",)
                        }
                    }
                } else {
                    match config::get_all_entries(&path, &db_password) {
                        Ok(entries) => {
                            for (account, Entry { username, password }) in entries {
                                println!("`{account}`:\n  {username}: {password}",)
                            }
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Failed to run command `get`. Err = {:?}",
                                e
                            ))
                        }
                    }
                }
            }
            Command::Init { name } => {
                if let Err(e) = config::create_empty_db(
                    &name,
                    &db_password,
                    std::env::current_dir()?,
                ) {
                    return Err(anyhow!(
                        "Failed to run command `init`. Err = {:?}",
                        e
                    ));
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
                )?;
            }
        },
        None => {}
    }

    Ok(())
}

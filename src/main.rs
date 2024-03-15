mod config;
mod store;

use std::path::Path;

use anyhow::anyhow;
use clap::{Parser, Subcommand};

pub type Result<T> = anyhow::Result<T>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Add {
        db: String,
        username: String,
        pass: String,
    },
    Get {
        db: String,
        username: String,
    },
    Init {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => match &cmd {
            Command::Add { db, username, pass } => {
                let path = std::env::current_dir()?.join(format!("{db}.pwm"));
                if let Err(e) = config::add_entry(&path, &username, &pass) {
                    return Err(anyhow!(
                        "Failed to run command `add`. Err = {:?}",
                        e
                    ));
                }
            }
            Command::Get { db, username } => {
                let path = std::env::current_dir()?.join(format!("{db}.pwm"));
                match config::get_entry(&path, username) {
                    Err(e) => {
                        return Err(anyhow!(
                            "Failed to run command `get`. Err = {:?}",
                            e
                        ))
                    }
                    Ok(pass) => println!("{username}: {pass}"),
                }
            }
            Command::Init { name } => {
                if let Err(e) =
                    config::create_empty_db(&name, std::env::current_dir()?)
                {
                    return Err(anyhow!(
                        "Failed to run command `init`. Err = {:?}",
                        e
                    ));
                }
            }
        },
        None => {}
    }

    Ok(())
}

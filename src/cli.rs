use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// If not supplied, pwm will use $PWM_PASS instead.
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add password to file.
    Add {
        /// The password file
        db_file: String,

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
        db_file: String,

        #[arg(short)]
        payload: String,
    },
    /// Print password from file. If `account` is not provided, prints all entries in db.
    Get {
        db_file: String,
        account: Option<String>,
    },
    /// Create an empty pwm file.
    Init { name: String },
}

#[derive(Serialize, Deserialize)]
pub struct JsonPayload {
    pub account: String,
    pub username: String,
    pub password: String,
}

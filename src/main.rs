mod app;
mod cli;
mod config;
mod db;
mod store;

use app::App;
use cli::Cli;

use clap::Parser;

pub type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = App::new(cli);

    app.run()
}

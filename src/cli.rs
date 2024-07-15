use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    pub config: PathBuf,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

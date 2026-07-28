use crate::parser::{manual::ManualCommand, profile::ProfileCommand};
use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub enum Cli {
    Manual(ManualCommand),

    /// Profile related commands
    #[command(subcommand)]
    Profile(ProfileCommand),
}

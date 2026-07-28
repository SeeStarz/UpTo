use clap::{Args, Subcommand};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct ManualArgs {
    /// Profile to override currently active one
    #[arg(long)]
    pub profile: Option<String>,

    /// Agent host:port to talk to. Overrides profile settings
    #[arg(long)]
    pub agent: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct ManualCommand {
    #[command(flatten)]
    pub args: ManualArgs,

    /// Manual command
    #[command(subcommand)]
    pub command: ManualCommandType,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Subcommand)]
pub enum ManualCommandType {
    /// Begin a new manual fact
    Begin(BeginCommand),
    /// End a manual fact
    End(EndCommand),
    /// End all facts
    EndAll,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct BeginCommand {
    pub title: String,
    /// Will be stored in key description in the metadata
    pub description: Option<String>,
    /// Valid json representing string to string hashmap. Will override description if key exists
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct EndCommand {
    pub title: String,
}

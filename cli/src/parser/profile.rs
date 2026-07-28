use clap::{Args, Subcommand};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Subcommand)]
pub enum ProfileCommand {
    /// Create new profile
    New(NewCommand),
    /// Edit existing profile
    Edit(EditCommand),
    /// Delete existing profile
    Delete(ProfileName),
    /// List profiles
    List,
    /// Show profile details
    Show(ProfileName),
    /// Use profile as default
    Use(ProfileName),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct ProfileName {
    /// Name of the profile
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct EditCommand {
    /// Profile name
    pub name: String,
    /// The agent IP and port to create and connect to
    #[arg(long)]
    pub agent_host: Option<String>,
    /// The server http(s) address for the agent to talk to
    #[arg(long)]
    pub server_address: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
pub struct NewCommand {
    /// Profile name
    pub name: String,
    /// The agent IP and port to create and connect to
    pub agent: String,
    /// The server http(s) address for the agent to talk to
    pub server: String,
}

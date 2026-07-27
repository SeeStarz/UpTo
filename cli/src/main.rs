use clap::{Args, Parser, Subcommand};
use common::{AgentFactCommand, serialize_wire_json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, net::TcpStream, path::Path, println};

#[derive(Parser)]
#[command(about)]
struct Cli {
    #[arg(long, default_value_t = String::from("localhost:7000"))]
    host: String,

    #[arg(long, short)]
    profile: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Begin a new manual fact
    Begin(Begin),
    /// End a manual fact
    End(End),
    /// End all facts
    EndAll,
    /// Profile related commands
    #[command(subcommand)]
    Profile(ProfileCommand),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Subcommand)]
enum ProfileCommand {
    /// Create new profile
    New(ProfileNew),
    /// Edit existing profile
    Edit(ProfileEdit),
    /// Delete existing profile
    Delete(ProfileDelete),
    /// List profiles
    List,
    /// Show profile details
    Show(ProfileShow),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct Begin {
    title: String,
    /// Will be stored in key description in the metadata
    description: Option<String>,
    /// Valid json representing string to string hashmap. Will override description if key exists
    #[arg(long)]
    json: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct End {
    title: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct ProfileNew {
    /// Profile name
    name: String,
    /// The agent IP and port to create and connect to
    agent_host: String,
    /// The server http(s) address for the agent to talk to
    server_address: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct ProfileEdit {
    /// Profile name
    name: String,
    /// The agent IP and port to create and connect to
    agent_host: Option<String>,
    /// The server http(s) address for the agent to talk to
    server_address: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct ProfileDelete {
    /// Profile name
    name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Args)]
struct ProfileShow {
    /// Profile name
    name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
struct Profile {
    agent_host: String,
    server_address: String,
}

fn send_command(command: &AgentFactCommand, stream: &mut TcpStream) {
    serialize_wire_json(stream, command).expect("Failed to send command");
}

fn main() {
    let cli = Cli::parse();
    let config_base = dirs::config_dir().unwrap().join("upto");
    let mut stream = TcpStream::connect(cli.host).expect("Failed to connect with agent");

    {
        use Commands::*;
        match cli.command {
            Begin(cmd) => {
                let mut map = HashMap::new();

                if let Some(json) = cmd.json {
                    map = serde_json::from_str::<HashMap<String, String>>(&json)
                        .expect("Failed to deserialize json");
                }

                if let Some(description) = cmd.description
                    && !map.contains_key("description")
                {
                    map.insert(String::from("description"), description);
                };

                let command = AgentFactCommand::Begin {
                    title: cmd.title,
                    metadata: map,
                };
                send_command(&command, &mut stream);
            }
            End(cmd) => {
                let command = AgentFactCommand::End { title: cmd.title };
                send_command(&command, &mut stream);
            }
            EndAll => {
                let command = AgentFactCommand::EndAll;
                send_command(&command, &mut stream);
            }
            Profile(cmd) => {
                handle_profile_cmd(cmd, &config_base);
            }
        };
    }
}

fn get_profiles(profile_dir: &Path) -> HashMap<String, Profile> {
    let profile_names: Vec<String> = fs::read_dir(&profile_dir)
        .map(|d| {
            d.filter_map(|r| r.ok())
                .filter_map(|r| r.file_name().to_str().map(|s| s.to_string()))
                .collect()
        })
        .expect("Failed to list profiles");

    let profiles = HashMap::from_iter(
        profile_names
            .iter()
            .flat_map(|name| {
                let content = fs::read(profile_dir.join(name));
                if content.is_ok() {
                    Some((name, content.unwrap()))
                } else {
                    None
                }
            })
            .filter_map(|(name, content)| {
                let data = serde_json::from_slice::<Profile>(&content);
                if data.is_ok() {
                    Some((name.clone(), data.unwrap()))
                } else {
                    None
                }
            }),
    );

    profiles
}

fn handle_profile_cmd(cmd: ProfileCommand, base_confdir: &Path) {
    let profile_confdir = base_confdir.join("profile");
    fs::create_dir_all(&profile_confdir).expect("Failed to create profile directory");

    use ProfileCommand::*;
    match cmd {
        New(cmd) => {
            let profile = Profile {
                agent_host: cmd.agent_host,
                server_address: cmd.server_address,
            };
            fs::write(
                &profile_confdir.join(cmd.name),
                &serde_json::to_vec(&profile).expect("Failed to serialize profile"),
            )
            .expect("Failed to write profile");
        }
        Edit(cmd) => {
            if let Some(mut profile) = get_profiles(&profile_confdir).remove(&cmd.name) {
                if let Some(agent_host) = cmd.agent_host {
                    profile.agent_host = agent_host;
                }
                if let Some(server_address) = cmd.server_address {
                    profile.server_address = server_address;
                }
                fs::write(
                    &profile_confdir.join(cmd.name),
                    &serde_json::to_vec(&profile).expect("Failed to serialize profile"),
                )
                .expect("Failed to edit profile");
            } else {
                println!("Profile not found");
            }
        }
        Delete(cmd) => {
            if let Some(_profile) = get_profiles(&profile_confdir).get(&cmd.name) {
                fs::remove_file(&profile_confdir.join(cmd.name)).expect("Failed to delete file");
            } else {
                println!("Profile not found");
            }
        }
        List => {
            let profiles = get_profiles(&profile_confdir);
            if profiles.len() == 0 {
                println!("No profile found");
            } else {
                println!("Profiles:");
                for name in profiles.keys() {
                    println!("- {}", name);
                }
            }
        }
        Show(cmd) => {
            if let Some(profile) = get_profiles(&profile_confdir).get(&cmd.name) {
                println!(
                    "Profile {}:\n{}",
                    cmd.name,
                    serde_json::to_string(profile).expect("Failed to serialize profile")
                );
            } else {
                println!("Profile not found");
            }
        }
    }
}

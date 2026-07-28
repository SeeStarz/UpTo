use crate::parser::export::{
    Cli, ManualCommand, ManualCommandType, Parser, ProfileCommand, ProfileNewCommand,
};
use common::{AgentFactCommand, serialize_wire_json};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    net::TcpStream,
    path::{Path, PathBuf},
    println,
};

mod parser;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
struct Profile {
    name: String,
    agent_host: String,
    server_address: String,
}

impl From<ProfileNewCommand> for Profile {
    fn from(value: ProfileNewCommand) -> Self {
        Profile {
            agent_host: value.agent,
            name: value.name,
            server_address: value.server,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Setting {
    config_dir: PathBuf,
    state_dir: PathBuf,
    profile_dir: PathBuf,
}

impl Default for Setting {
    fn default() -> Self {
        let config_dir = dirs::config_dir().unwrap().join("upto");
        let state_dir = dirs::state_dir().unwrap().join("upto");
        let profile_dir = config_dir.join("profile");
        Self {
            config_dir,
            state_dir,
            profile_dir,
        }
    }
}

impl Setting {
    fn init(&self) {
        fs::create_dir_all(&self.config_dir).expect("Failed to create config directory");
        fs::create_dir_all(&self.state_dir).expect("Failed to create state directory");
        fs::create_dir_all(&self.profile_dir).expect("Failed to create profile directory");
    }
}

fn send_command(command: &AgentFactCommand, agent_host: &str) {
    let mut stream = TcpStream::connect(agent_host).expect("Failed to connect to agent");
    serialize_wire_json(&mut stream, command).expect("Failed to send command");
}

fn main() {
    let cli = Cli::parse();

    let setting = Setting::default();
    setting.init();

    {
        use Cli::*;
        match cli {
            Manual(cmd) => {
                handle_manual(cmd, &setting);
            }

            Profile(cmd) => {
                handle_profile_cmd(cmd, &setting);
            }
        };
    }
}

fn get_active_profile(setting: &Setting) -> Option<Profile> {
    let contents = fs::read(setting.state_dir.join("active_profile.json"));

    let Ok(contents) = contents else {
        return None;
    };

    let Ok(profile_name) = serde_json::from_slice::<String>(&contents) else {
        eprintln!("Failed to deserialize active_profile.json");
        return None;
    };

    get_profiles(&setting.profile_dir)
        .get(&profile_name)
        .cloned()
}

fn get_profile_final(setting: &Setting, profile_arg: Option<&String>) -> Option<Profile> {
    let profiles = get_profiles(&setting.profile_dir);

    profile_arg
        .and_then(|name| profiles.get(name))
        .map(|p| p.clone())
        .or(get_active_profile(setting))
}

fn handle_manual(manual_command: ManualCommand, setting: &Setting) {
    let profile = get_profile_final(setting, manual_command.args.profile.as_ref());
    let agent_host = manual_command
        .args
        .agent
        .or(profile.map(|p| p.agent_host))
        .expect("Failed to deduce agent host. Try configuring a profile");

    {
        use ManualCommandType::*;
        match manual_command.command {
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
                send_command(&command, &agent_host);
            }
            End(cmd) => {
                let command = AgentFactCommand::End { title: cmd.title };
                send_command(&command, &agent_host);
            }
            EndAll => {
                let command = AgentFactCommand::EndAll;
                send_command(&command, &agent_host);
            }
        }
    }
}

fn get_profiles(profile_dir: &Path) -> HashMap<String, Profile> {
    let profile_names: Vec<String> = fs::read_dir(&profile_dir)
        .map(|d| {
            d.filter_map(|r| r.ok())
                .filter_map(|r| {
                    r.file_name()
                        .to_str()
                        .and_then(|s| s.strip_suffix(".json").map(|s| s.to_string()))
                })
                .collect()
        })
        .expect("Failed to list profiles");

    let profiles = HashMap::from_iter(
        profile_names
            .iter()
            .flat_map(|name| {
                fs::read(profile_dir.join(format!("{}.json", name)))
                    .ok()
                    .map(|c| (name, c))
            })
            .filter_map(|(name, content)| {
                serde_json::from_slice::<Profile>(&content)
                    .ok()
                    .map(|d| (name.clone(), d))
            })
            .filter(|(name, profile)| *name == profile.name),
    );

    profiles
}

fn handle_profile_cmd(cmd: ProfileCommand, setting: &Setting) {
    use ProfileCommand::*;
    match cmd {
        New(cmd) => {
            let profile = Profile::from(cmd);
            fs::write(
                &setting.profile_dir.join(format!("{}.json", &profile.name)),
                &serde_json::to_vec(&profile).expect("Failed to serialize profile"),
            )
            .expect("Failed to write profile");
        }
        Edit(cmd) => {
            if let Some(mut profile) = get_profiles(&setting.profile_dir).remove(&cmd.name) {
                if let Some(agent_host) = cmd.agent_host {
                    profile.agent_host = agent_host;
                }
                if let Some(server_address) = cmd.server_address {
                    profile.server_address = server_address;
                }
                fs::write(
                    &setting.profile_dir.join(format!("{}.json", cmd.name)),
                    &serde_json::to_vec(&profile).expect("Failed to serialize profile"),
                )
                .expect("Failed to edit profile");
            } else {
                println!("Profile not found");
            }
        }
        Delete(cmd) => {
            if let Some(_profile) = get_profiles(&setting.profile_dir).get(&cmd.name) {
                fs::remove_file(&setting.profile_dir.join(format!("{}.json", cmd.name)))
                    .expect("Failed to delete file");
            } else {
                println!("Profile not found");
            }
        }
        List => {
            let profiles = get_profiles(&setting.profile_dir);
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
            if let Some(profile) = get_profiles(&setting.profile_dir).get(&cmd.name) {
                println!(
                    "Profile {}:\n{}",
                    cmd.name,
                    serde_json::to_string(profile).expect("Failed to serialize profile")
                );
            } else {
                println!("Profile not found");
            }
        }
        Use(cmd) => fs::write(
            setting.state_dir.join("active_profile.json"),
            &serde_json::to_vec(&cmd.name).expect("Failed to serialize profile name"),
        )
        .expect("Failed to set default profile"),
    }
}

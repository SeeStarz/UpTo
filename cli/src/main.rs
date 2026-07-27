use std::{collections::HashMap, net::TcpStream};

use clap::{Parser, Subcommand};
use common::{AgentFactCommand, serialize_wire_json};

#[derive(Parser)]
#[command(about)]
struct Cli {
    #[arg(long, default_value_t = String::from("localhost:7000"))]
    host: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Begin a new manual fact
    Begin {
        title: String,
        /// Will be stored in key description in the metadata
        description: Option<String>,
        /// Valid json representing string to string hashmap. Will override description if key exists
        #[arg(long)]
        json: Option<String>,
    },

    /// End a manual fact
    End { title: String },

    /// End all facts
    EndAll,
}

fn send_command(command: AgentFactCommand, stream: TcpStream) {
    serialize_wire_json(stream, command).expect("Failed to send command");
}

fn main() {
    let cli = Cli::parse();

    let stream = TcpStream::connect(cli.host).expect("Failed to connect with agent");

    {
        use Commands::*;
        match cli.command {
            Begin {
                title,
                description,
                json,
            } => {
                let mut map = HashMap::new();

                if let Some(json) = json {
                    map = serde_json::from_str::<HashMap<String, String>>(&json)
                        .expect("Failed to deserialize json");
                }

                if let Some(description) = description
                    && !map.contains_key("description")
                {
                    map.insert(String::from("description"), description);
                };

                let command = AgentFactCommand::Begin {
                    title,
                    metadata: map,
                };
                send_command(command, stream);
            }
            End { title } => {
                let command = AgentFactCommand::End { title };
                send_command(command, stream);
            }
            EndAll => {
                let command = AgentFactCommand::EndAll;
                send_command(command, stream);
            }
        };
    }
}

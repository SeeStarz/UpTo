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
    },

    /// End a manual fact
    End { title: String },

    /// End all facts
    EndAll,
}

fn send_command(command: AgentFactCommand, stream: TcpStream) {
    serialize_wire_json(stream, command).expect("Unable to send command");
}

fn main() {
    let cli = Cli::parse();

    let stream = TcpStream::connect(cli.host).expect("Unable to connect with agent");

    {
        use Commands::*;
        match cli.command {
            Begin { title, description } => {
                let mut map = HashMap::new();
                if let Some(description) = description {
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

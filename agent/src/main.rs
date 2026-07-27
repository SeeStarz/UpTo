use common::{AgentFactCommand, Fact, Snapshot, deserialize_wire_json};
use std::{
    collections::HashMap,
    eprintln, fs,
    net::TcpListener,
    println,
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let (tx, rx) = mpsc::channel();

    let _cli_server = thread::spawn(move || cli_handler(tx));

    let mut facts = HashMap::new();

    if let Ok(data) = fs::read(dirs::state_dir().unwrap().join("upto/cache.json")) {
        if let Ok(cached_facts) = serde_json::from_slice(&data) {
            facts = cached_facts;
        } else {
            eprintln!("Failed to deserialize cached facts");
        }
    }

    loop {
        for agent_fact_command in rx.try_iter() {
            use AgentFactCommand::*;
            match agent_fact_command {
                Begin { title, metadata } => {
                    facts.insert(
                        title.clone(),
                        Fact {
                            observer_name: String::from("Manual"),
                            observation_title: title,
                            metadata: metadata,
                        },
                    );
                }
                End { title } => {
                    facts.remove(&title);
                }
                EndAll => {
                    facts.drain();
                }
            }
        }
        let snapshot = Snapshot {
            facts: facts.clone().into_values().collect(),
        };

        fs::create_dir_all(dirs::state_dir().unwrap().join("upto"))
            .expect("Failed to create state directory");
        fs::write(
            dirs::state_dir().unwrap().join("upto/cache.json"),
            &serde_json::to_vec(&facts).expect("Failed to serialize facts"),
        )
        .expect("Failed to write state file");

        let client = reqwest::blocking::Client::new();
        if let Err(err) = client
            .post("http://localhost:8000/api")
            .json(&snapshot)
            .send()
        {
            println!("{:?}", err);
        } else {
            println!("Okie");
        }
        sleep(Duration::from_millis(5000));
    }
}

fn cli_handler(sender: mpsc::Sender<AgentFactCommand>) {
    let listener = TcpListener::bind("localhost:7000").expect("Failed to bind");
    loop {
        let Ok((stream, _addr)) = listener.accept() else {
            eprintln!("Failed to accept connection request");
            continue;
        };

        let Ok(data) = deserialize_wire_json::<AgentFactCommand, _>(stream) else {
            eprintln!("Failed to deserialize incoming data");
            continue;
        };

        let Ok(_) = sender.send(data) else {
            eprintln!("Failed to send data to channel");
            continue;
        };
    }
}

use common::{AgentFactCommand, Fact, Snapshot, deserialize_wire_json};
use reqwest::header;
use std::{
    collections::HashMap,
    env, eprintln, fs, io,
    net::TcpListener,
    println,
    sync::{Arc, Mutex, mpsc},
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let shutdown_requested = Arc::new(Mutex::new(false));
    {
        let var = shutdown_requested.clone();
        ctrlc::set_handler(move || *var.lock().unwrap() = true)
            .expect("Failed to set signal handler");
    }

    let (tx, rx) = mpsc::channel();
    {
        let var = shutdown_requested.clone();
        let _cli_server = thread::spawn(move || cli_handler(tx, var));
    }

    let mut facts = HashMap::new();
    if let Ok(data) = fs::read(dirs::state_dir().unwrap().join("upto/cache.json")) {
        if let Ok(cached_facts) = serde_json::from_slice(&data) {
            facts = cached_facts;
        } else {
            eprintln!("Failed to deserialize cached facts, starting anew");
        }
    }

    let admin_token = env::var("ADMIN_TOKEN").unwrap_or(String::from("admin_btw"));

    loop {
        if *shutdown_requested.lock().expect("Failed to acquire lock") {
            break;
        }

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
        match client
            .post("http://localhost:8000/api")
            .header(header::AUTHORIZATION, format!("Custom {}", admin_token))
            .json(&snapshot)
            .send()
        {
            Err(err) => {
                eprintln!("Failed to send request {:?}", err);
            }
            Ok(response) => match response.error_for_status() {
                Ok(_response) => {
                    println!("Sent!")
                }
                Err(err) => {
                    eprintln!("Request failed {:?}", err);
                }
            },
        }
        sleep(Duration::from_millis(1000));
    }
}

fn cli_handler(sender: mpsc::Sender<AgentFactCommand>, shutdown_requested: Arc<Mutex<bool>>) {
    let listener = TcpListener::bind("localhost:7000").expect("Failed to bind");
    listener
        .set_nonblocking(true)
        .expect("Failed to set listener nonblocking");
    loop {
        if *shutdown_requested.lock().expect("Failed to acquire lock") {
            break;
        }

        let stream = match listener.accept() {
            Ok((stream, _addr)) => stream,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                eprintln!("Failed to accept connection request {:?}", e);
                continue;
            }
        };

        let Ok(data) = deserialize_wire_json::<AgentFactCommand, _>(stream) else {
            eprintln!("Failed to deserialize incoming data");
            continue;
        };

        let Ok(_) = sender.send(data) else {
            eprintln!("Failed to send data to channel");
            continue;
        };

        sleep(Duration::from_millis(1000));
    }
}

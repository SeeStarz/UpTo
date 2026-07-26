use common::{AgentFactCommand, Fact, Snapshot, deserialize_wire_json};
use std::{
    collections::HashMap,
    eprintln,
    net::TcpListener,
    println,
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let _cli_server = thread::spawn(cli_handler);

    loop {
        let fact = Fact {
            metadata: HashMap::new(),
            observer_name: String::from("Manual"),
            observation_title: String::from("working on upto"),
        };
        let snapshot = Snapshot { facts: vec![fact] };
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

fn cli_handler() {
    let listener = TcpListener::bind("localhost:7000").expect("Failed to bind");
    loop {
        if let Ok((stream, _addr)) = listener.accept() {
            if let Ok(data) = deserialize_wire_json::<AgentFactCommand, _>(stream) {
                println!("{:?}", data);
            } else {
                eprintln!("Failed to deserialize incoming data");
            }
        } else {
            eprintln!("Failed to accept connection request");
        }
    }
}

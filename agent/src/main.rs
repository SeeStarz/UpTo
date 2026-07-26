use common::{Fact, Snapshot};
use std::{collections::HashMap, println, thread::sleep, time::Duration};

fn main() {
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

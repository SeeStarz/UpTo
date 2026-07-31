use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Fact {
    pub observer_name: String,
    pub observation_title: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Snapshot {
    pub facts: Vec<Fact>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum AgentFactCommand {
    Begin {
        title: String,
        metadata: HashMap<String, String>,
    },
    End {
        title: String,
    },
    EndAll,
}

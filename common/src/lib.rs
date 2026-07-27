use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

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

pub fn serialize_wire_json<W, T>(writer: &mut W, data: T) -> io::Result<()>
where
    W: Write,
    T: Serialize,
{
    let data = serde_json::to_vec(&data)?;
    writer.write_all(&(data.len() as u32).to_be_bytes())?;
    writer.write_all(&data)?;
    io::Result::Ok(())
}

pub fn deserialize_wire_json<T, R>(reader: &mut R) -> io::Result<T>
where
    T: DeserializeOwned,
    R: Read,
{
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let len = u32::from_be_bytes(buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let data = serde_json::from_slice(&buf)?;
    io::Result::Ok(data)
}

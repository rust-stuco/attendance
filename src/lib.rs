use clap::Args;
use serde::{Deserialize, Serialize};

pub mod cli;
pub mod manager;

#[derive(Args, Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Student {
    /// Student's Andrew ID.
    #[arg(short, long = "aid")]
    pub andrew_id: String,

    /// Student's name. Since this is not strictly necessary for management, it is optional.
    #[arg(short, long)]
    pub name: Option<String>,
}

impl Student {
    pub fn new(andrew_id: String, name: Option<String>) -> Self {
        Self { andrew_id, name }
    }

    pub fn email(&self) -> String {
        format!("{}@andrew.cmu.edu", self.andrew_id)
    }
}

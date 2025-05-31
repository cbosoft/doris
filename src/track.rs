use std::collections::HashMap;
use std::fs::OpenOptions;

use ratatui::Frame;
use serde::{Serialize, Deserialize};

use crate::patch::Patch;
use crate::sequence::Sequence;

#[derive(Serialize, Deserialize)]
pub struct Track {
    bpm: f32,
    patches: HashMap<String, Patch>,
    sequences: HashMap<String, Sequence>,
    play_order: Vec<(String, f32)>, // sequence name, length (seconds)
}


impl Track {
    pub fn new() -> Self {
        Track { bpm: 140.0, patches: HashMap::new(), sequences: HashMap::new(), play_order: Vec::new() }
    }

    pub fn from_file(p: &str) -> anyhow::Result<Self> {
        let f = OpenOptions::new().read(true).open(p)?;
        let rv = serde_yaml::from_reader(f)?;
        Ok(rv)
    }

    pub fn draw_sequence_list(&self, frame: &mut Frame) {
    }

    pub fn draw_patch_list(&self, frame: &mut Frame) {
    }
}

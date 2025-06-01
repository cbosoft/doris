use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SequenceLayer {
    divisions: usize,
    patch: String,
    notes: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    layers: HashMap<String, SequenceLayer>
}

impl Sequence {
    pub fn new() -> Self {
        Self { layers: HashMap::new() }
    }
}

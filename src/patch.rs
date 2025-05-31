use std::{collections::HashMap, fs::OpenOptions};

use serde::{Serialize, Deserialize};

use fundsp::hacker::*;

#[derive(Serialize, Deserialize)]
#[serde(tag="op")]
pub enum PatchNode {
    Constant { c: f32 },
    Sine,
    Saw,
    Square,
    SpecifiedSine { freq: f32 },
    SpecifiedSaw { freq: f32 },
    SpecifiedSquare { freq: f32 },
    Pan { balance: f32 },
}

impl PatchNode {
    pub fn add_to_net(&self, net: &mut Net) -> NodeId {
        match self {
            Self::Constant { c }            => { net.push(Box::new(constant(*c))) },
            Self::Sine                      => { net.push(Box::new(sine())) },
            Self::Saw                       => { net.push(Box::new(saw())) },
            Self::Square                    => { net.push(Box::new(square())) },
            Self::SpecifiedSine { freq }    => { net.push(Box::new(sine_hz(*freq))) },
            Self::SpecifiedSaw { freq }     => { net.push(Box::new(saw_hz(*freq))) },
            Self::SpecifiedSquare { freq }  => { net.push(Box::new(square_hz(*freq))) },
            Self::Pan { balance }           => { net.push(Box::new(pan(*balance))) },
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Patch {
    nodes: HashMap<String, PatchNode>,
    edges: HashMap<String, String>
}

impl Patch {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), edges: HashMap::new() }
    }

    pub fn from_file(p: &str) -> anyhow::Result<Self> {
        let f = OpenOptions::new().read(true).open(p)?;
        let rv = serde_yaml::from_reader(f)?;
        Ok(rv)
    }

    fn create_subnet(&self, from: &str, to: &str) -> Net {
        let mut net = Net::new(1, 1);
        let mut current = from.to_string();
        let mut current_node_id = None;
        net.pass_through(0, 0);

        loop {
            match self.edges.get(&current) {
                Some(next) if next == to => {
                    eprintln!("{current} -> {next} (end)");
                    break;
                }
                Some(next) => {
                    eprintln!("{current} -> {next}");
                    let node = self.nodes.get(next).unwrap();
                    let next_node_id = node.add_to_net(&mut net);

                    if let Some(current_node_id) = current_node_id {
                        net.pipe_all(current_node_id, next_node_id);
                    }
                    else {
                        net.pipe_input(next_node_id);
                    }

                    net.pipe_output(next_node_id);
                    current = next.clone();
                    current_node_id = Some(next_node_id);
                }
                None => {
                    panic!();
                }
            }
        }
        net
    }

    pub fn create_nets(&self) -> (Net, Net) {
        let freq_net = self.create_subnet("freq", "out");
        //let freq_net = unit::<U1, U1>(Box::new(freq_net));
        //let freq_net = net.push(Box::new(freq_net));
        //net.set_source(freq_net, 0, Source::Global(0));

        let ctl_net = self.create_subnet("ctl", "out");
        //let ctl_net = unit::<U1, U1>(Box::new(ctl_net));
        //let ctl_net = net.push(Box::new(ctl_net));
        //net.set_source(ctl_net, 1, Source::Global(1));

        //let pass_node = net.push(Box::new(multipass()));
        //net.set_source(pass_node, 0, Source::Global(1));

        //net = net >> (freq_net | ctl_net) >> 
        //net.check();

        (freq_net, ctl_net)
    }
}

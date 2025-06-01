use std::{collections::HashMap, fs::OpenOptions, sync::Arc};

use serde::{Serialize, Deserialize};

use fundsp::hacker::*;

#[derive(Serialize, Deserialize)]
#[serde(tag="op")]
pub enum PatchNode {
    Constant { c: f32 },

    // Oscillators
    Sine,
    Saw,
    Square,
    SpecifiedSine { freq: f32 },
    SpecifiedSaw { freq: f32 },
    SpecifiedSquare { freq: f32 },

    // Sample
    Sample { path: String },

    // Noise
    WhiteNoise,
    PinkNoise,
    BrownNoise,

    // Effects
    FlangerSin { strength: f32, min_delay: f32, max_delay: f32, sin_freq: f32 },
    ADSR { attack: f32, decay: f32, sustain: f32, release: f32 },

    // Filters
    // TODO

    // Maths
    SumChannels,
    MultChannels,
    Mux,

    // Pan { balance: f32 },
}

impl PatchNode {
    pub fn add_to_net(&self, net: &mut Net) -> anyhow::Result<NodeId> {
        let rv = match self {
            Self::Constant { c }            => { net.push(Box::new(constant(*c))) },

            // Oscillators
            Self::Sine                      => { net.push(Box::new(sine())) },
            Self::Saw                       => { net.push(Box::new(saw())) },
            Self::Square                    => { net.push(Box::new(square())) },
            Self::SpecifiedSine { freq }    => { net.push(Box::new(sine_hz(*freq))) },
            Self::SpecifiedSaw { freq }     => { net.push(Box::new(saw_hz(*freq))) },
            Self::SpecifiedSquare { freq }  => { net.push(Box::new(square_hz(*freq))) },

            // Sample
            Self::Sample { path }           => {
                let wave = Wave::load(path)?;
                let wave = Arc::new(wave);
                net.push(Box::new(
                    wavech(&wave, 0, Some(0))
                ))
            }

            // Maths
            Self::SumChannels               => { net.push(Box::new( map(|f: &Frame<f32, U2>| f[0]+f[1]) )) },
            Self::MultChannels | Self::Mux  => { net.push(Box::new( map(|f: &Frame<f32, U2>| f[0]*f[1]) )) },

            // Noise
            Self::WhiteNoise                => { net.push(Box::new( white() )) },
            Self::PinkNoise                 => { net.push(Box::new( pink() )) },
            Self::BrownNoise                => { net.push(Box::new( brown() )) },

            // Effects
            Self::FlangerSin { strength, min_delay, max_delay, sin_freq } => { 
                let strength = strength.clone(); let min_delay = min_delay.clone(); let max_delay = max_delay.clone(); let sin_freq = sin_freq.clone();
                net.push(Box::new(
                    flanger(strength, min_delay, max_delay, move |t| lerp11(min_delay, max_delay, sin_hz(sin_freq, t)))
                ))
            },
            Self::ADSR { attack, decay, sustain, release } => {
                net.push(Box::new( adsr_live(*attack, *decay, *sustain, *release) ))
            }

            // Filters
            // TODO

            // Self::Pan { balance }           => { net.push(Box::new(pan(*balance))) },
            _ => { todo!() }
        };

        Ok(rv)
    }
}


#[derive(Serialize, Deserialize)]
pub struct Patch {
    nodes: HashMap<String, PatchNode>,
    edges: Vec<(String, String)>
}

impl Patch {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("osc1".into(), PatchNode::Saw);
        nodes.insert("flanger".into(), PatchNode::FlangerSin { strength: 0.5, min_delay: 0.005, max_delay: 0.01, sin_freq: 0.1 });
        nodes.insert("adsr".into(), PatchNode::ADSR { attack: 1.0, decay: 0.5, sustain: 0.5, release: 0.5 });
        nodes.insert("mux".into(), PatchNode::Mux);
        let mut edges = Vec::new();
        edges.push(("freq".into(), "osc1".into()));
        edges.push(("osc1".into(), "flanger".into()));
        edges.push(("flanger".into(), "mux:0".into()));
        edges.push(("ctl".into(), "adsr".into()));
        edges.push(("adsr".into(), "mux:1".into()));
        edges.push(("mux".into(), "out".into()));
        Self { nodes, edges }
    }

    pub fn from_file(p: &str) -> anyhow::Result<Self> {
        let f = OpenOptions::new().read(true).open(p)?;
        let rv = serde_yaml::from_reader(f)?;
        Ok(rv)
    }

    fn get_branches(end: String, edges: &Vec<(String, String)>) -> anyhow::Result<Vec<Vec<String>>> {
        let mut rv = Vec::new();

        for (i, o) in edges.iter() {
            let (i, _) = Self::parse_node_name(i)?;
            let (o, _) = Self::parse_node_name(o)?;

            if o == end {
                let child_branches = Self::get_branches(i.clone(), edges)?;
                if child_branches.len() == 0 {
                    rv.push(vec![i.clone(), o.clone()]);
                }
                for mut child in child_branches.into_iter() {
                    child.push(o.clone());
                    rv.push(child);
                }
            }
        }

        Ok(rv)
    }

    pub fn branch_reprs(&self) -> anyhow::Result<Vec<String>> {
        let branches = Self::get_branches("out".into(), &self.edges)?;
        Ok(branches.into_iter().map(|b| b.join("-->")).collect())
    }

    fn parse_node_name(n: &String) -> anyhow::Result<(String, usize)> {
        let rv = match n.split_once(":") {
            Some((n, ch)) => (n.to_string(), ch.parse()?),
            None => (n.to_string(), 0)
        };
        Ok(rv)
    }

    pub fn create_net(&self) -> anyhow::Result<Net> {
        let mut net = Net::new(2, 1);
        net.pass_through(0, 0);

        let mut nodes_by_id = HashMap::new();
        for (node_name, node) in self.nodes.iter() {
            let node_id = node.add_to_net(&mut net)?;
            nodes_by_id.insert(node_name.clone(), node_id);
        }

        for (src, snk) in self.edges.iter() {
            let (src, src_ch) = Self::parse_node_name(src)?;
            let (snk, snk_ch) = Self::parse_node_name(snk)?;
            let src_id = nodes_by_id.get(&src).cloned();
            let snk_id = nodes_by_id.get(&snk).cloned();
            // eprintln!("{src} -> {snk}");

            match (src.as_str(), snk.as_str()) {
                ("freq", "out") => {
                    net.pass_through(0, 0);
                    // eprintln!("pass freq:0 to out:0");
                },
                ("ctl", "out") => {
                    net.pass_through(1, 0);
                    // eprintln!("pass ctl:0 to out:0");
                },
                ("freq", _) => {
                    if let Some(snk_id) = snk_id {
                        net.set_source(snk_id, snk_ch, Source::Global(0));
                        // eprintln!("pass freq:0 to {snk}:{snk_ch}");
                    } // TODO: otherwise error
                },
                ("ctl", _) => {
                    if let Some(snk_id) = snk_id {
                        net.set_source(snk_id, snk_ch, Source::Global(1));
                        // eprintln!("pass ctl:0 to {snk}:{snk_ch}");
                    } // TODO: otherwise error
                },
                (_, "out") => {
                    if let Some(src_id) = src_id {
                        net.pipe_output(src_id);
                        // eprintln!("pass {src}: to out:");
                    }
                },
                _ => {
                    if let Some((src_id, snk_id)) = src_id.zip(snk_id) {
                        net.set_source(snk_id, snk_ch, Source::Local(src_id, src_ch));
                        // eprintln!("pass {src}:{src_ch} to {snk}:{snk_ch}");
                    }
                },
            }

        }
        Ok(net)
    }

    // pub fn create_net(&self) -> anyhow::Result<Net> {
    //     let freq_net = self.create_subnet("freq", "out")?;
    //     //let freq_net = unit::<U1, U1>(Box::new(freq_net));
    //     //let freq_net = net.push(Box::new(freq_net));
    //     //net.set_source(freq_net, 0, Source::Global(0));

    //     let ctl_net = self.create_subnet("ctl", "out")?;
    //     //let ctl_net = unit::<U1, U1>(Box::new(ctl_net));
    //     //let ctl_net = net.push(Box::new(ctl_net));
    //     //net.set_source(ctl_net, 1, Source::Global(1));

    //     //let pass_node = net.push(Box::new(multipass()));
    //     //net.set_source(pass_node, 0, Source::Global(1));

    //     //net = net >> (freq_net | ctl_net) >> 
    //     //net.check();

    //     Ok((freq_net, ctl_net))
    // }
}

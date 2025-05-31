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
    Mux,
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
            Self::Mux                       => { todo!(); }
            Self::Pan { balance }           => { net.push(Box::new(pan(*balance))) },
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Patch {
    nodes: HashMap<String, PatchNode>,
    edges: Vec<(String, String)>
}

impl Patch {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), edges: Vec::new() }
    }

    pub fn from_file(p: &str) -> anyhow::Result<Self> {
        let f = OpenOptions::new().read(true).open(p)?;
        let rv = serde_yaml::from_reader(f)?;
        Ok(rv)
    }

    fn get_branches(end: String, edges: &Vec<(String, String)>) -> Vec<Vec<String>> {
        let mut rv = Vec::new();

        for (i, o) in edges.iter() {
            if o == &end {
                let child_branches = Self::get_branches(i.clone(), edges);
                if child_branches.len() == 0 {
                    rv.push(vec![i.clone(), o.clone()]);
                }
                for mut child in child_branches.into_iter() {
                    child.push(o.clone());
                    rv.push(child);
                }
            }
        }

        rv
    }

    pub fn branch_reprs(&self) -> Vec<String> {
        let branches = Self::get_branches("out".into(), &self.edges);
        branches.into_iter().map(|b| b.join("-->")).collect()
    }

    fn create_subnet(&self, from: &str, to: &str) -> anyhow::Result<Net> {
        let mut net = Net::new(1, 1);
        net.pass_through(0, 0);

        let mut nodes_by_id = HashMap::new();
        for (node_name, node) in self.nodes.iter() {
            let node_id = node.add_to_net(&mut net);
            nodes_by_id.insert(node_name.clone(), node_id);
        }

        for (src, snk) in self.edges.iter() {
            let src_id = nodes_by_id.get(src).cloned();
            let snk_id = nodes_by_id.get(snk).cloned();

            match (src, snk) {
                (from, to) if (from == from) && (to == to) => {
                    eprintln!("{from} -> {to}");
                    net.pass_through(0, 0);
                },
                (from, snk) if from == from => {
                    eprintln!("{from} -> {snk}");
                    net.pipe_input(snk_id.unwrap());
                },
                (src, to) if to == to => {
                    eprintln!("{src} -> {to}");
                    net.pipe_output(src_id.unwrap());
                },
                (src, snk) => {
                    eprintln!("{src} -> {snk}");
                    net.pipe_all(src_id.unwrap(), snk_id.unwrap());
                },
            }

        }
        Ok(net)
    }

    pub fn create_nets(&self) -> anyhow::Result<(Net, Net)> {
        let freq_net = self.create_subnet("freq", "out")?;
        //let freq_net = unit::<U1, U1>(Box::new(freq_net));
        //let freq_net = net.push(Box::new(freq_net));
        //net.set_source(freq_net, 0, Source::Global(0));

        let ctl_net = self.create_subnet("ctl", "out")?;
        //let ctl_net = unit::<U1, U1>(Box::new(ctl_net));
        //let ctl_net = net.push(Box::new(ctl_net));
        //net.set_source(ctl_net, 1, Source::Global(1));

        //let pass_node = net.push(Box::new(multipass()));
        //net.set_source(pass_node, 0, Source::Global(1));

        //net = net >> (freq_net | ctl_net) >> 
        //net.check();

        Ok((freq_net, ctl_net))
    }
}

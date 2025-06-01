use std::collections::HashMap;
use std::time::Duration;
use std::io::{Write, stdout};

use crossterm::execute;
use crossterm::event::{
    KeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags
};
use fundsp::funutd::Rnd;
use fundsp::hacker::*;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Tabs, Widget}, Frame};
use crossterm::{event, event::*};
use crossterm::event::{KeyCode, KeyEventKind};

use crate::{command_box::CommandBox, event_handler::EventHandler, patch::Patch, sequence::Sequence, track::Track};
use crate::keyboard::{Keyboard, Note, NoteEvent, NoteEventKind};
use crate::frame_renderable::FrameRenderable;


#[derive(Debug)]
pub enum Arg {
    None,
    Path(String),
    PatchName,
    NewPatchName,
    SequenceName,
    NewSequenceName,
    // TODO: others
}

#[derive(Debug)]
enum AppCommand {
    Exit, Play,
    LoadTrack(String),
    LoadPatch(String),
    LoadSequence(String),
    EditPatch(String),
    CreatePatch(String),
    CreateSequence(String)
        // TODO: others
}

impl AppCommand {
    fn list_commands() -> Vec<(String, Arg)> {
        vec![
            ("exit".into(), Arg::None),
            ("play".into(), Arg::None),
            ("load track".into(), Arg::Path("*.yaml".into())),
            ("load patch".into(), Arg::Path("*.yaml".into())),
            ("load sequence".into(), Arg::Path("*.yaml".into())),
            ("create patch".into(), Arg::NewPatchName),
            ("edit patch".into(), Arg::PatchName),
            ("create sequence".into(), Arg::NewSequenceName),
            ("edit sequence".into(), Arg::SequenceName),
        ]
    }
}

impl TryFrom<&str> for AppCommand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(" ").map(|v|v).collect();
        let p1 = parts.get(0).cloned();
        let p2 = parts.get(1).cloned();
        let p3 = parts.get(2).cloned();
        match (p1, p2, p3) {
            (Some("exit"), None, None) => Ok(AppCommand::Exit),
            (Some("play"), None, None) => Ok(AppCommand::Play),
            (Some("load"), Some("track"), Some(s)) => Ok(AppCommand::LoadTrack(s.into())),
            _ => Err(format!("unrecognised command \"{value}\""))
        }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Command,
    Play,
}


pub struct App {
    rng: Rnd,
    net: Net,
    seq: Sequencer,
    seq_events: HashMap<(Note, i32), EventId>,
    track: Track,
    patch: Patch,
    sequence: Sequence,
    cbox: CommandBox,
    kb: Keyboard,
    mode: Mode,
}


impl App {
    pub fn new(mut net: Net, sample_rate: f64) -> Self {
        let mut seq = Sequencer::new(false, 1);
        seq.set_sample_rate(sample_rate);
        net.chain(Box::new(seq.backend()));
        net.chain(Box::new(pan(0.0)));
        net.commit();

        let mut cbox = CommandBox::new();
        cbox.set_autocomplete(AppCommand::list_commands());
        Self {
            rng: Rnd::from_u64(0),
            cbox,
            kb: Keyboard::new(),
            track: Track::new(),
            patch: Patch::new(),
            sequence: Sequence::new(),
            net,
            seq,
            seq_events: HashMap::new(),
            mode: Mode::Play
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut term = ratatui::init();

        let mut stdout = stdout();
        execute!(stdout, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)).unwrap();
        execute!(stdout, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)).unwrap();

        loop {
            term.draw(|f| {
                self.draw(f);
            })?;

            if self.process_events()? {
                break;
            }

            let should_stop = match self.mode {
                Mode::Command => {
                    self.run_mode_command()
                }
                Mode::Play => {
                    self.run_mode_play()
                }
            }?;
            if should_stop {
                break;
            }

        }

        Ok(())
    }

    fn run_mode_command(&mut self) -> anyhow::Result<bool> {
        self.cbox.update_autocomplete();

        if let Some(cmd) = self.cbox.get_command() {
            // run command
            match AppCommand::try_from(cmd.as_str()) {
                Ok(cmd) => {
                    match cmd {
                        AppCommand::Exit => {
                            return Ok(true);
                        },
                        AppCommand::LoadTrack(path) => {
                            match Track::from_file(&path) {
                                Ok(track) => {
                                    self.track = track;
                                    self.cbox.push_output(format!("Loaded track from \"{path}\"."));
                                }
                                Err(_) => {
                                    self.cbox.push_error(format!("Failed to load track from \"{path}\"."));
                                }
                            }
                        },
                        AppCommand::LoadPatch(path) => {
                            match Patch::from_file(&path) {
                                Ok(patch) => {
                                    self.patch = patch;
                                    self.cbox.push_output(format!("Loaded patch from \"{path}\"."));
                                }
                                Err(_) => {
                                    self.cbox.push_error(format!("Failed to load track from \"{path}\"."));
                                }
                            }
                        },
                        AppCommand::Play => {
                            self.mode = Mode::Play;
                        }
                        _ => {
                            self.cbox.push_error(format!("Unhandled cmd {cmd:?}"));
                        }
                    }
                },
                Err(m) => {
                    self.cbox.push_error(format!("Error: {m}"));
                }
            }
        }

        Ok(false)
    }

    fn run_mode_play(&mut self) -> anyhow::Result<bool> {
        if self.kb.is_finished() {
            self.mode = Mode::Command;
        }

        let events = self.kb.get_events();
        // TODO
        for event in events {
            match event {
                NoteEvent { kind: NoteEventKind::Start, note, octave } => {
                    let k = (note, octave);
                    if !self.seq_events.contains_key(&k) {
                        let f = note.to_freq_octave(octave);
                        let pnet = self.patch.create_net().unwrap();
                        let pnet = unit::<U2, U1>(Box::new(pnet));
                        let mut unit = Box::new(
                            (constant(f) | constant(1.0)) >> pnet
                        );
                        unit.ping(false, AttoHash::new(self.rng.u64()));
                        let event_id = self.seq.push_relative(
                            0.0,
                            f64::INFINITY,
                            Fade::Power,
                            0.0,
                            0.0,
                            unit,
                        );
                        self.seq_events.insert(k, event_id);
                    }
                },
                NoteEvent { kind: NoteEventKind::Stop, note, octave } => {
                    if let Some(event_id) = self.seq_events.remove(&(note, octave)) {
                        self.seq.edit_relative(event_id, 0.0, 0.0);
                    }
                },
            }
        }

        Ok(false)
    }

    fn selected<'a>(&'a mut self) -> &'a mut dyn EventHandler {
        match self.mode {
            Mode::Command => &mut self.cbox,
            Mode::Play => &mut self.kb,
        }
    }

    fn process_events(&mut self) -> anyhow::Result<bool> {
        if !event::poll(Duration::from_millis(16))? {
            Ok(false)
        }
        else {
            let selected = self.selected();
            match event::read()? {
                Event::Key(kev) => {
                    match kev {
                        KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press, ..} => {
                            Ok(true)
                        },
                        KeyEvent { code: KeyCode::Up, kind: KeyEventKind::Press, ..} => {
                            Ok(false)
                        },
                        KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, ..} => {
                            Ok(false)
                        },
                        kev => selected.handle_key(kev)
                    }
                },
                Event::Resize(c, r) => selected.handle_resize(r, c),
                Event::Mouse(mev) => selected.handle_mouse(mev),
                Event::FocusLost => selected.handle_focus_lost(),
                Event::FocusGained => selected.handle_focus_gain(),
                Event::Paste(s) => selected.handle_paste(s),
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
        let mut stdout = stdout();
        let _ = execute!(stdout, PopKeyboardEnhancementFlags);
        let _ = execute!(stdout, PopKeyboardEnhancementFlags);
    }
}


impl FrameRenderable for App {
    fn draw_into(&self, frame: &mut Frame, area: Rect) {
        let [workspace, _, bottom] = Layout::new(Direction::Vertical, vec![
                Constraint::Min(50),
                Constraint::Length(1),
                Constraint::Min(25),
            ])
            .areas(area)
            ;


        let [_, cli, _] = Layout::new(Direction::Horizontal, vec![
                Constraint::Min(0),
                Constraint::Percentage(50),
                Constraint::Min(0),
            ])
            .areas(bottom);

        // let [_, tab_area] = Layout::new(Direction::Horizontal, vec![
        //     Constraint::Length(3), Constraint::Min(20)
        // ]).areas(workspace);
        // let block1 = Block::new().borders(Borders::ALL);
        // let workspace_area = block1.inner(workspace);
        // block1.render(workspace, frame.buffer_mut());
        // let tabs = Tabs::new(vec!["1/Patch", "2/Sequence", "3/Play"]);
        // tabs.render(tab_area, frame.buffer_mut());

        match self.mode {
            Mode::Command => { self.cbox.draw_into(frame, bottom); },
            Mode::Play => { self.kb.draw_into(frame, bottom); }
        }
    }
}

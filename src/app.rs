use std::time::Duration;

use fundsp::hacker::*;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Tabs, Widget}, Frame};
use crossterm::{event, event::*};
use crossterm::event::{KeyCode, KeyEventKind};

use crate::{command_box::CommandBox, event_handler::EventHandler, patch::Patch, sequence::Sequence, track::Track};
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
    Exit,
    LoadTrack(String),
    EditPatch(String),
    CreatePatch(String),
    CreateSequence(String)
        // TODO: others
}

impl AppCommand {
    fn list_commands() -> Vec<(String, Arg)> {
        vec![
            ("exit".into(), Arg::None),
            ("load track".into(), Arg::Path("*.yaml".into())),
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
            (Some("load"), Some("track"), Some(s)) => Ok(AppCommand::LoadTrack(s.into())),
            _ => Err(format!("unrecognised command \"{value}\""))
        }
    }
}


pub struct App {
    net: Net,
    track: Track,
    patch: Patch,
    sequence: Sequence,
    cbox: CommandBox,
}


impl App {
    pub fn new(net: Net) -> Self {
        let mut cbox = CommandBox::new();
        cbox.set_autocomplete(AppCommand::list_commands());
        Self { cbox, track: Track::new(), patch: Patch::new(), sequence: Sequence::new(), net, mode: Mode::Command }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut term = ratatui::init();

        loop {
            term.draw(|f| {
                self.draw(f);
            })?;

            if self.process_events()? {
                break;
            }

            self.cbox.update_autocomplete();

            if let Some(cmd) = self.cbox.get_command() {
                // run command
                match AppCommand::try_from(cmd.as_str()) {
                    Ok(cmd) => {
                        match cmd {
                            AppCommand::Exit => {
                                break;
                            },
                            AppCommand::LoadTrack(path) => {
                                match Track::from_file(&path) {
                                    Ok(track) => {
                                        self.track = track;
                                        self.cbox.push_output(format!("Loaded track from {path}."));
                                    }
                                    Err(e) => {
                                        self.cbox.push_error(format!("Failed to load track from {path}."));
                                    }
                                }
                            },
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
        }

        Ok(())
    }

    fn selected<'a>(&'a mut self) -> &'a mut dyn EventHandler {
        &mut self.cbox
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
    }
}


impl FrameRenderable for App {
    fn draw_into(&self, frame: &mut Frame, area: Rect) {
        let [workspace, _, bottom] = Layout::new(Direction::Vertical, vec![
                Constraint::Min(50),
                Constraint::Length(1),
                Constraint::Max(10),
            ])
            .areas(area)
            ;


        let [_, cli, _] = Layout::new(Direction::Horizontal, vec![
                Constraint::Min(1),
                Constraint::Percentage(50),
                Constraint::Min(1),
            ])
            .areas(bottom);

        let [_, tab_area] = Layout::new(Direction::Horizontal, vec![
            Constraint::Length(3), Constraint::Min(20)
        ]).areas(workspace);

        let block1 = Block::new().borders(Borders::ALL);
        let workspace_area = block1.inner(workspace);
        block1.render(workspace, frame.buffer_mut());
        let tabs = Tabs::new(vec!["1/Patch", "2/Sequence", "3/Play"]);
        tabs.render(tab_area, frame.buffer_mut());

        self.cbox.draw_into(frame, cli);
    }
}

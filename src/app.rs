use std::time::Duration;

use fundsp::shared::Shared;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Widget}, Frame};
use crossterm::{event, event::*};
use crossterm::event::{KeyCode, KeyEventKind};
use strum::{EnumIter, IntoEnumIterator};

use crate::{command_box::CommandBox, event_handler::EventHandler};
use crate::frame_renderable::FrameRenderable;

#[derive(EnumIter)]
enum AppCommand {
    PitchUp,
    PitchDown
}

impl AppCommand {
    fn list_commands() -> Vec<String> {
        Self::iter().map(|v| v.into()).collect()
    }
}

impl TryFrom<&str> for AppCommand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pitch up" => Ok(Self::PitchUp),
            "pitch down" => Ok(Self::PitchDown),
            _ => Err(format!("unrecognised command \"{value}\""))
        }
    }
}

impl From<AppCommand> for String {
    fn from(value: AppCommand) -> Self {
        match value {
            AppCommand::PitchUp => "pitch up".into(),
            AppCommand::PitchDown => "pitch down".into(),
        }
    }
}

pub struct App {
    cbox: CommandBox,
    pitch: Shared,
}


impl App {
    pub fn new(pitch: Shared) -> Self {
        let mut cbox = CommandBox::new();
        cbox.set_autocomplete(AppCommand::list_commands());
        Self { cbox, pitch }
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

            if let Some(cmd) = self.cbox.get_command() {
                // run command
                match AppCommand::try_from(cmd.as_str()) {
                    Ok(cmd) => {
                        match cmd {
                            AppCommand::PitchUp => {
                                self.pitch_up();
                            },
                            AppCommand::PitchDown => {
                                self.pitch_down();
                            },
                        }
                    },
                    Err(m) => {
                        self.cbox.push_error(format!("Error: \"{m}\""));
                    }
                }
            }
        }

        Ok(())
    }

    fn pitch_up(&mut self) {
        let new_pitch = (self.pitch.value() + 10.0).min(500.0);
        self.pitch.set(new_pitch);
    }

    fn pitch_down(&mut self) {
        let new_pitch = (self.pitch.value() - 10.0).max(50.0);
        self.pitch.set(new_pitch);
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
                            self.pitch_up();
                            Ok(false)
                        },
                        KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, ..} => {
                            self.pitch_down();
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
        let outer = Layout::new(Direction::Vertical, vec![
                Constraint::Percentage(70),
                Constraint::Max(1),
                Constraint::Min(10),
            ])
            .split(area)
            ;

        let inner = Layout::new(Direction::Horizontal, vec![
                Constraint::Min(1),
                Constraint::Percentage(50),
                Constraint::Min(1),
            ])
            .split(outer[2]);

        let block1 = Block::new().borders(Borders::ALL);
        block1.render(outer[0], frame.buffer_mut());

        self.cbox.draw_into(frame, inner[1]);
    }
}

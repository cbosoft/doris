use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Rectangle, Shape};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::prelude::*;

use crate::event_handler::EventHandler;
use crate::frame_renderable::FrameRenderable;


pub enum NoteEventKind {
    Start, Stop
}

pub struct NoteEvent {
    kind: NoteEventKind,
}

pub struct Keyboard {
    notes: [bool;25],
    events: Vec<NoteEvent>,
    octave: i32,
    finished: bool
}

impl Keyboard {
    pub fn new() -> Self {
        Self { notes: [false;25], events: Vec::new(), octave: 3, finished: false }
    }

    pub fn get_events(&mut self) -> Vec<NoteEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

impl EventHandler for Keyboard {
    fn handle_key(&mut self, kev: KeyEvent) -> anyhow::Result<bool> {
        match kev {
            KeyEvent { code: KeyCode::Esc, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, .. } => {
                self.finished = true;
            },
            KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind, .. } => {
                let j = match c {
                    'q' => Some(0),  // C
                    '2' => Some(1),  // C#
                    'w' => Some(2),  // D
                    '3' => Some(3),  // D#
                    'e' => Some(4),  // E/Fb
                    'r' => Some(5),  // F/E#
                    '5' => Some(6),  // F#
                    't' => Some(7),  // G
                    '6' => Some(8),  // G#
                    'y' => Some(9),  // A
                    '7' => Some(10), // A#
                    'u' => Some(11), // B

                    'c' => Some(12), // C
                    'f' => Some(13), // C#
                    'v' => Some(14), // D
                    'g' => Some(15), // D#
                    'b' => Some(16), // E
                    'n' => Some(17), // F
                    'j' => Some(18), // F#
                    'm' => Some(19), // G
                    'k' => Some(20), // G#
                    ',' => Some(21), // A
                    'l' => Some(22), // A#
                    '.' => Some(23), // B
                                     
                    '/' => Some(24), // C
                    _ => None
                };

                if let Some(j) = j {
                    self.notes[j] = matches!(kind, KeyEventKind::Press);
                }
            },
            _ => ()
        }

        Ok(false)
    }
}

impl Shape for Keyboard {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let cw = Color::White;
        let cb = Color::DarkGray;
        let ww = 4;
        let wh = 15;
        let bw = 3;
        let bh = 7;
        let g = 1;

        let mut j = 0;
        for i in 0..15 {
            let n = i % 7;
            let jb = if (n != 0) && (n != 3) {
                let oj = j;
                j += 1;
                Some(oj)
            }
            else {
                None
            };

            if j >= self.notes.len() { break; }
            let dy: usize = if self.notes[j] { 1 } else { 0 };
            for x in 0..ww {
                for y in 0..wh {
                    let x = x + (ww+g)*i;
                    painter.paint(x, y + dy, cw);
                }
            }
            j += 1;

            if let Some(j) = jb {
                if j >= self.notes.len() { break; }
                let dy: usize = if self.notes[j] { 1 } else { 0 };
                for x in 0..bw {
                    for y in 0..bh {
                        let x = x + (ww+g)*i - (bw/2) - g;
                        painter.paint(x, y+dy, cb);
                    }
                }
            }

        }
    }
}

impl FrameRenderable for Keyboard {
    fn draw_into(&self, frame: &mut Frame, area: Rect) {
        // series of keys on a keyboard
        let n = (self.notes.len() / 12) as u16 * 7 + 1;
        let l = (n*4) + (n-1);

        let [_, kb_area, _] = Layout::new(Direction::Horizontal, vec![
            Constraint::Min(0),
            Constraint::Length(l),
            Constraint::Min(0),
        ]).areas(area);
        let [_, kb_area, _] = Layout::new(Direction::Vertical, vec![
            Constraint::Min(0),
            Constraint::Length(16),
            Constraint::Min(0),
        ]).areas(kb_area);

        Canvas::default()
            .x_bounds([0.0, 108.0])
            .y_bounds([0.0, 20.0])
            .marker(Marker::Block)
            .paint(|ctx| {
                ctx.draw(self);
            })
            .render(kb_area, frame.buffer_mut());
    }
}

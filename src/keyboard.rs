use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Rectangle, Shape};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::prelude::*;

use crate::event_handler::EventHandler;
use crate::frame_renderable::FrameRenderable;


struct KB_Key {
    pub x: u16,
    pub y: u16,
    pub is_black_key: bool,
    pub is_dim: bool
}

impl KB_Key {
    const WHITE_KEY_WIDTH: u16 = 4;
    const WHITE_KEY_HEIGHT: u16 = 15;
    const BLACK_KEY_WIDTH: u16 = 3;
    const BLACK_KEY_HEIGHT: u16 = 7;

    pub fn get_size(&self) -> (u16, u16) {
        if self.is_black_key {
            (Self::BLACK_KEY_WIDTH, Self::BLACK_KEY_HEIGHT)
        }
        else {
            (Self::WHITE_KEY_WIDTH, Self::WHITE_KEY_HEIGHT)
        }
    }

    pub fn get_colour(&self) -> Color {
        if self.is_dim {
            if self.is_black_key {
                Color::Black
            }
            else {
                Color::DarkGray
            }
        }
        else {
            if self.is_black_key {
                Color::DarkGray
            }
            else {
                Color::White
            }
        }
    }
}

impl Shape for KB_Key {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let colour = self.get_colour();
        let (w, h) = self.get_size();
        for dx in 0..w {
            for dy in 0..h {
                painter.paint((self.x+dx) as usize, (self.y + dy) as usize, colour);
            }
        }
    }
}


pub enum NoteEventKind {
    Start, Stop
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Note {
    C, CSharp, D, DSharp, E, F, FSharp, G, GSharp, A, ASharp, B, 
}

impl Note {
    pub fn to_freq(&self) -> f32 {
        // in 3rd
        match self {
            Self::C => 130.8,
            Self::CSharp => 138.6,
            Self::D => 146.8,
            Self::DSharp => 155.6,
            Self::E => 164.8,
            Self::F => 174.6,
            Self::FSharp => 185.0,
            Self::G => 196.0,
            Self::GSharp => 207.7,
            Self::A => 220.0,
            Self::ASharp => 233.1,
            Self::B => 246.9,
        }
    }

    pub fn to_freq_octave(&self, o: i32) -> f32 {
        let m = 2f32.powi(o - 3);
        self.to_freq()*m
    }

    pub fn from_index(i: usize) -> Self {
         match i % 12 {
            0 => Self::C,
            1 => Self::CSharp,
            2 => Self::D,
            3 => Self::DSharp,
            4 => Self::E,
            5 => Self::F,
            6 => Self::FSharp,
            7 => Self::G,
            8 => Self::GSharp,
            9 => Self::A,
            10 => Self::ASharp,
            11 => Self::B, 
            _ => { panic!() }
        }
    }
}

pub struct NoteEvent {
    pub kind: NoteEventKind,
    pub note: Note,
    pub octave: i32,
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

    fn clear_notes(&mut self) {
        for i in 0..self.notes.len() {
            if self.notes[i] {
                let note_index = i % 12;
                let note = Note::from_index(note_index);
                let octave = self.octave + (i as i32 / 12);
                self.events.push(NoteEvent { kind: NoteEventKind::Stop, note, octave });
            }
        }
    }

    pub fn incr_octave(&mut self) {
        self.clear_notes();
        self.octave = (self.octave + 1).min(7);
    }

    pub fn decr_octave(&mut self) {
        self.clear_notes();
        self.octave = (self.octave - 1).max(-2);
    }
}

impl EventHandler for Keyboard {
    fn handle_key(&mut self, kev: KeyEvent) -> anyhow::Result<bool> {
        match kev {
            KeyEvent { code: KeyCode::Esc, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, .. } => {
                self.finished = true;
            },
            KeyEvent { code: KeyCode::Char('>'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, .. } => { self.incr_octave(); },
            KeyEvent { code: KeyCode::Char('<'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, .. } => { self.decr_octave(); },
            KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind, .. } => {
                let data = match c {
                    'q' => Some((0,  Note::C,      0)),
                    '2' => Some((1,  Note::CSharp, 0)),
                    'w' => Some((2,  Note::D,      0)),
                    '3' => Some((3,  Note::DSharp, 0)),
                    'e' => Some((4,  Note::E,      0)),
                    'r' => Some((5,  Note::F,      0)),
                    '5' => Some((6,  Note::FSharp, 0)),
                    't' => Some((7,  Note::G,      0)),
                    '6' => Some((8,  Note::GSharp, 0)),
                    'y' => Some((9,  Note::A,      0)),
                    '7' => Some((10, Note::ASharp, 0)),
                    'u' => Some((11, Note::B,      0)),

                    'c' => Some((12, Note::C,      1)),
                    'f' => Some((13, Note::CSharp, 1)),
                    'v' => Some((14, Note::D,      1)),
                    'g' => Some((15, Note::DSharp, 1)),
                    'b' => Some((16, Note::E,      1)),
                    'n' => Some((17, Note::F,      1)),
                    'j' => Some((18, Note::FSharp, 1)),
                    'm' => Some((19, Note::G,      1)),
                    'k' => Some((20, Note::GSharp, 1)),
                    ',' => Some((21, Note::A,      1)),
                    'l' => Some((22, Note::ASharp, 1)),
                    '.' => Some((23, Note::B,      1)),

                    '/' => Some((24, Note::C,      2)),
                    _ => None
                };

                if let Some((j, note, doctave)) = data {
                    match kind {
                        KeyEventKind::Press => {
                            self.notes[j] = true;
                            self.events.push(NoteEvent {
                                kind: NoteEventKind::Start,
                                note,
                                octave: self.octave + doctave
                            });
                        },
                        KeyEventKind::Release => {
                            self.notes[j] = false;
                            self.events.push(NoteEvent {
                                kind: NoteEventKind::Stop,
                                note,
                                octave: self.octave + doctave
                            });
                        },
                        _ => ()
                    }
                }
            },
            _ => ()
        }

        Ok(false)
    }
}

impl Shape for Keyboard {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let ww = KB_Key::WHITE_KEY_WIDTH;
        let bw = KB_Key::BLACK_KEY_WIDTH;
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
            let dy: u16 = if self.notes[j] { 1 } else { 0 };
            KB_Key { x: (ww+g)*i, y: dy, is_black_key: false, is_dim: false}.draw(painter);
            j += 1;

            if let Some(j) = jb {
                if j >= self.notes.len() { break; }
                let dy: u16 = if self.notes[j] { 1 } else { 0 };
                KB_Key { x: (ww+g)*i - (bw/2) - g, y: dy, is_black_key: true, is_dim: false}.draw(painter);
            }
        }
    }
}

impl FrameRenderable for Keyboard {
    fn draw_into(&self, frame: &mut Frame, area: Rect) {
        // series of keys on a keyboard
        let n = (self.notes.len() / 12) as u16 * 7 + 1;
        let octave_length = 7*KB_Key::WHITE_KEY_WIDTH + 11;

        let [_, kb_area, _] = Layout::new(Direction::Horizontal, vec![
            Constraint::Min(0),
            Constraint::Length(2*octave_length),
            Constraint::Min(0),
        ]).areas(area);
        let [_, kb_area, _] = Layout::new(Direction::Vertical, vec![
            Constraint::Length(1),
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

        frame.buffer_mut().set_string(kb_area.x, kb_area.y-1, format!("{}", self.octave), Style::new().dark_gray());
        frame.buffer_mut().set_string(kb_area.x + octave_length - 4, kb_area.y-1, format!("{}", self.octave+1), Style::new().dark_gray());
        frame.buffer_mut().set_string(kb_area.x + octave_length*2 - 8, kb_area.y-1, format!("{}", self.octave+2), Style::new().dark_gray());
    }
}

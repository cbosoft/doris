use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::prelude::*;

use crate::event_handler::EventHandler;
use crate::frame_renderable::FrameRenderable;

enum HistoryType {
    Output, Echo, Error
}

pub struct CommandBox {
    ac: Vec<String>,
    history: VecDeque<(HistoryType, String)>,
    buf: String,
    ready: bool,
    cursor_position: usize,
}

impl CommandBox {
    pub fn new() -> Self {
        Self { ac: Vec::new(), history: VecDeque::new(), buf: String::new(), ready: false, cursor_position: 0 }
    }

    pub fn get_command(&mut self) -> Option<String> {
        if self.ready {
            let cmd = std::mem::take(&mut self.buf);
            self.cursor_position = 0;
            self.ready = false;
            Some(cmd)
        }
        else {
            None
        }
    }

    pub fn push_output(&mut self, output: String) {
        self.push(HistoryType::Output, output);
    }

    pub fn push_error(&mut self, error: String) {
        self.push(HistoryType::Error, error);
    }

    fn echo(&mut self) {
        self.push(HistoryType::Echo, self.buf.clone());
    }

    fn push(&mut self, ht: HistoryType, s: String) {
        self.history.push_back((ht, s));
        while self.history.len() > 200 {
            let _ = self.history.pop_front();
        }
    }

    pub fn set_autocomplete(&mut self, options: Vec<String>) {
        self.ac = options;
    }

    fn autocomplete(&self) -> String {
        if self.buf.len() == 0 {
            return String::new();
        }

        for opt in self.ac.iter() {
            if opt.starts_with(&self.buf) {
                let opt: Vec<_> = opt.chars().rev().take(opt.len() - self.buf.len()).collect();
                let opt: String = opt.into_iter().rev().collect();
                return opt;
            }
        }
        String::new()
    }
}


impl EventHandler for CommandBox {
    fn handle_key(&mut self, kev: KeyEvent) -> anyhow::Result<bool> {
        match kev {
            KeyEvent { code: KeyCode::Char(c), kind: KeyEventKind::Press, .. } => {
                if self.cursor_position == self.buf.len() {
                    self.buf.push(c);
                }
                else {
                    self.buf.insert(self.cursor_position, c);
                }
                self.cursor_position += 1;
            },
            KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. } => {
                self.echo();
                self.ready = true;
            },
            KeyEvent { code: KeyCode::Esc, kind: KeyEventKind::Press, .. } => {
                self.buf = String::new();
                self.cursor_position = 0;
            },
            KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. } => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
            },
            KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. } => {
                self.cursor_position = self.cursor_position.saturating_add(1).min(self.buf.len());
            },
            KeyEvent { code: KeyCode::Tab, kind: KeyEventKind::Press, .. } => {
                let ac = self.autocomplete();
                self.buf += &ac;
                self.cursor_position = self.buf.len();
            },
            KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. } => {
                if (self.cursor_position > 0) && (self.buf.len() > 0) {
                    self.buf.remove(self.cursor_position-1);
                    self.cursor_position = self.cursor_position.saturating_sub(1);
                }
            }
            KeyEvent { code: KeyCode::Delete, kind: KeyEventKind::Press, .. } => {
                if self.cursor_position < self.buf.len() {
                    self.buf.remove(self.cursor_position);
                }
            }
            _ => (),
        };

        Ok(false)
    }
}


impl FrameRenderable for CommandBox {
    fn draw_into(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new().borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());
        let area = inner;

        let mut lines: VecDeque<_> = self.history.iter().map(|(ht, s)| {
            let (content_style, prefix) = match ht {
                HistoryType::Echo => (Style::new(), ">> "),
                HistoryType::Output => (Style::new().blue(), "   "),
                HistoryType::Error => (Style::new().red(), "!  "),
            };
            Line::from(vec![
                Span::styled(prefix, Style::new().dim()),
                Span::styled(s, content_style)
            ])
        }).collect();
        let resp_h = area.height as usize;
        while lines.len() < resp_h {
            lines.push_front(Line::from(""));
        }
        while lines.len() >= resp_h {
            lines.pop_front();
        }

        let line = Line::from(vec![
            Span::styled(">> ", Style::new().dim()),
            Span::raw(&self.buf),
            Span::styled(self.autocomplete(), Style::new().dark_gray()),
        ]);

        lines.push_back(line);
        let lines: Vec<_> = lines.into_iter().collect();
        let para = Paragraph::new(lines);
        para.render(area, frame.buffer_mut());

        let cx = area.x + 3 + (self.cursor_position as u16);
        let cy = area.y + area.height - 1;
        frame.set_cursor_position((cx, cy));
    }
}

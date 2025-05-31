use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::prelude::*;

use crate::event_handler::EventHandler;
use crate::frame_renderable::FrameRenderable;

pub struct CommandBox {
    response: VecDeque<String>,
    buf: String,
    ready: bool,
    cursor_position: usize,
}

impl CommandBox {
    pub fn new() -> Self {
        Self { response: VecDeque::new(), buf: String::new(), ready: false, cursor_position: 0 }
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

    pub fn push_response(&mut self, resp: String) {
        self.response.push_back(format!("<< {resp}"));
        while self.response.len() > 200 {
            let _ = self.response.pop_front();
        }
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
                self.response.push_back(format!(">> {}", self.buf));
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

        let mut lines: VecDeque<_> = self.response.iter().map(|r| Line::from(r.clone())).collect();
        let resp_h = area.height as usize;
        while lines.len() < resp_h {
            lines.push_front(Line::from(""));
        }
        while lines.len() >= resp_h {
            lines.pop_front();
        }

        let line = Line::from(format!(">> {}", self.buf));
        lines.push_back(line);
        let lines: Vec<_> = lines.into_iter().collect();
        let para = Paragraph::new(lines);
        para.render(area, frame.buffer_mut());

        let cx = area.x + 3 + (self.cursor_position as u16);
        let cy = area.y + area.height - 1;
        frame.set_cursor_position((cx, cy));
    }
}

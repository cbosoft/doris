use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::prelude::*;

use crate::app::Arg;
use crate::event_handler::EventHandler;
use crate::frame_renderable::FrameRenderable;

enum HistoryType {
    Output, Echo, Error
}

pub struct CommandBox {
    ac: Vec<(String, Arg)>,
    ac_suggestions: Vec<(String, String)>,
    ac_selection: usize,
    history: VecDeque<(HistoryType, String)>,
    buf: String,
    ready: bool,
    cursor_position: usize,
}

impl CommandBox {
    pub fn new() -> Self {
        Self {
            ac: Vec::new(),
            ac_suggestions: Vec::new(),
            ac_selection: 0,
            history: VecDeque::new(),
            buf: String::new(),
            ready: false,
            cursor_position: 0,
        }
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

    pub fn set_autocomplete(&mut self, options: Vec<(String, Arg)>) {
        self.ac = options;
    }

    pub fn update_autocomplete(&mut self) {
        let mut suggestions = Vec::new();
        for (stem, arg) in self.ac.iter() {
            if self.buf.starts_with(stem) || (&self.buf == stem) {
                // complete command, suggest values for args
                // TODO
                match arg {
                    Arg::None => { },
                    Arg::NewPatchName|Arg::SequenceName|Arg::PatchName|Arg::NewSequenceName => { suggestions.push((format!("{stem} "), format!("$name"))) },
                    Arg::Path(patt) => {suggestions.push((format!("{stem} "), format!("$path/{patt}"))) }
                    _ => { /*TODO*/ }
                }
            }
            else if stem.starts_with(&self.buf) {
                // incomplete command, suggest commands + arg proto
                match arg {
                    Arg::None => { suggestions.push((stem.clone(), String::new())) },
                    Arg::NewPatchName|Arg::SequenceName|Arg::PatchName|Arg::NewSequenceName => { suggestions.push((format!("{stem} "), format!("$name"))) },
                    Arg::Path(patt) => {suggestions.push((format!("{stem} "), format!("$path/{patt}"))) }
                    _ => { /*TODO*/ }
                }
            }
        }
        self.ac_suggestions = suggestions;
    }

    fn ac_next(&mut self) {
        let n_suggest = self.ac_suggestions.len();
        if n_suggest > 0 {
            let mut i = self.ac_selection.saturating_add(1);
            if i >= n_suggest {
                i = 0;
            }
            self.ac_selection = i;
        }
        else {
            self.ac_selection = 0;
        }
    }

    fn ac_select(&mut self) {
        self.buf = self.ac_suggestions[self.ac_selection].0.clone();
        self.cursor_position = self.buf.len();
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
                if self.cursor_position == self.buf.len() {
                    self.ac_select();
                }
                else {
                    self.cursor_position = self.cursor_position.saturating_add(1);
                }
            },
            KeyEvent { code: KeyCode::Tab, kind: KeyEventKind::Press, .. } => {
                self.ac_next();
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
        let block = Block::new().borders(Borders::ALL).title_bottom(Line::from("Tab for options; right arrow to select; enter to run.").centered());
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

        let n_suggest = self.ac_suggestions.len();
        let ac = if n_suggest > 0 {
            let (s, a) = self.ac_suggestions[self.ac_selection.min(n_suggest - 1)].clone();
            let n = s.chars().count();
            let s = if n > self.buf.len() {
                let chars: Vec<_> = s.chars().rev().take(n - self.buf.chars().count()).collect();
                chars.into_iter().rev().collect()
            }
            else {
                String::new()
            };
            format!("{s}{a}")
        }
        else {
            String::new()
        };

        let line = Line::from(vec![
            Span::styled(">> ", Style::new().dim()),
            Span::raw(&self.buf),
            Span::styled(ac, Style::new().dark_gray()),
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

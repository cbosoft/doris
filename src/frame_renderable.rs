use ratatui::{layout::Rect, Frame};

pub trait FrameRenderable {
    fn draw(&self, frame: &mut Frame) { self.draw_into(frame, frame.area()) }
    fn draw_into(&self, frame: &mut Frame, area: Rect);
}

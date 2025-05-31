use crossterm::event::{KeyEvent, MouseEvent};

pub trait EventHandler {
    fn handle_key(&mut self, kev: KeyEvent) -> anyhow::Result<bool> { let _ = kev; Ok(false) }
    fn handle_mouse(&mut self, mev: MouseEvent) -> anyhow::Result<bool> { let _ = mev; Ok(false) }
    fn handle_paste(&mut self, s: String) -> anyhow::Result<bool> { let _ = s; Ok(false) }
    fn handle_focus_lost(&mut self) -> anyhow::Result<bool> { Ok(false) }
    fn handle_focus_gain(&mut self) -> anyhow::Result<bool> { Ok(false) }
    fn handle_resize(&mut self, r: u16, c: u16) -> anyhow::Result<bool> { let _ = (r, c); Ok(false) }
}

impl<T: EventHandler + ?Sized> EventHandler for &mut T {
    fn handle_key(&mut self, kev: KeyEvent) -> anyhow::Result<bool> { (**self).handle_key(kev) }
    fn handle_mouse(&mut self, mev: MouseEvent) -> anyhow::Result<bool> { (**self).handle_mouse(mev) }
    fn handle_paste(&mut self, s: String) -> anyhow::Result<bool> { (**self).handle_paste(s) }
    fn handle_focus_lost(&mut self) -> anyhow::Result<bool> { (**self).handle_focus_lost() }
    fn handle_focus_gain(&mut self) -> anyhow::Result<bool> { (**self).handle_focus_gain() }
    fn handle_resize(&mut self, r: u16, c: u16) -> anyhow::Result<bool> { (**self).handle_resize(r, c) }
}

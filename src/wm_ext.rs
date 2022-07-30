use penrose::{core::ring::Direction, core::xconnection::XConn, Selector, WindowManager};

pub trait WindowManagerExt {
    /// Moves the focused window to the next screen in the given direction.
    ///
    /// # Errors
    /// Errors if an inner penrose command fails.
    fn cycle_client_to_screen(&mut self, direction: Direction) -> penrose::Result<()>;
}

impl<X: XConn> WindowManagerExt for WindowManager<X> {
    fn cycle_client_to_screen(&mut self, direction: Direction) -> penrose::Result<()> {
        let focused_index = match self.focused_client_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let current_screen = self.active_screen_index();

        match direction {
            Direction::Forward => {
                if current_screen == self.n_screens() - 1 {
                    self.client_to_screen(&Selector::Index(0))?;
                } else {
                    self.client_to_screen(&Selector::Index(current_screen + 1))?;
                }
            }
            Direction::Backward => {
                if current_screen == 0 {
                    self.client_to_screen(&Selector::Index(self.n_screens() - 1))?;
                } else {
                    self.client_to_screen(&Selector::Index(current_screen - 1))?;
                }
            }
        }

        self.focus_client(&Selector::WinId(focused_index))?;

        Ok(())
    }
}

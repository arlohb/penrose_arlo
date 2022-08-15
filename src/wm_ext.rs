use std::collections::HashMap;

use penrose::{
    common::geometry::Region, core::ring::Direction, xconnection::XConn, Selector, WindowManager,
};

use crate::{SwitchDirection, CLIENT_REGIONS};

const fn region_center(region: &Region) -> (usize, usize) {
    (
        (region.x + (region.w / 2)) as usize,
        (region.y + (region.h / 2)) as usize,
    )
}

pub trait WindowManagerExt {
    /// Moves the focused window to the next screen in the given direction.
    ///
    /// # Errors
    /// Errors if an inner penrose command fails.
    fn cycle_client_to_screen(&mut self, direction: Direction) -> penrose::Result<()>;

    /// Switches the focus in a physical direction, instead of in the stack.
    ///
    /// # Errors
    /// Errors if an inner penrose command fails.
    fn switch_focus_in_direction(&mut self, direction: SwitchDirection) -> penrose::Result<()>;
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

    fn switch_focus_in_direction(&mut self, direction: SwitchDirection) -> penrose::Result<()> {
        const fn dst_in_one_dir(a: usize, b: usize) -> usize {
            if a > b {
                std::usize::MAX
            } else {
                b - a
            }
        }

        let current_client = match self.focused_client_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let regions = CLIENT_REGIONS
            .read()
            .map_err(|_| penrose::Error::Raw("CLIENT_REGIONS RwLock poisoned".into()))?
            .clone();

        let current = region_center(&regions[&current_client]);

        let positions = regions
            .into_iter()
            .filter(|(id, _region)| *id != current_client)
            .map(|(id, region)| (id, region_center(&region)))
            .collect::<HashMap<_, _>>();

        let next = match direction {
            SwitchDirection::Up => {
                let on_same_x = positions
                    .iter()
                    .filter(|(_id, pos)| pos.0 == current.0)
                    .collect::<HashMap<_, _>>();

                if on_same_x.is_empty() {
                    todo!()
                } else {
                    let _ = on_same_x
                        .into_iter()
                        .min_by(|(_, a_c), (_, b_c)| {
                            dst_in_one_dir(a_c.1, current.1).cmp(&dst_in_one_dir(b_c.1, current.1))
                        })
                        .ok_or_else(|| unreachable!());
                    todo!()
                }
            }
            SwitchDirection::Down => positions.iter().max_by(|(_, a_center), (_, b_center)| {
                (a_center.1 - current.1).cmp(&(b_center.1 - current.1))
            }),
            SwitchDirection::Left => positions.iter().min_by(|(_, a_center), (_, b_center)| {
                (a_center.0 - current.0).cmp(&(b_center.0 - current.0))
            }),
            SwitchDirection::Right => positions.iter().max_by(|(_, a_center), (_, b_center)| {
                (a_center.0 - current.0).cmp(&(b_center.0 - current.0))
            }),
        }
        .map(|(&id, &pos)| (id, pos))
        .expect("No windows");

        self.focus_client(&Selector::WinId(next.0))?;

        Ok(())
    }
}

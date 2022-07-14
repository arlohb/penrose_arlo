use penrose::{
    core::{data_types::Point, xconnection::XConn, Hook},
    Selector,
};

pub struct NewWindowHook {}

impl NewWindowHook {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl<X: XConn> Hook<X> for NewWindowHook {
    fn new_client(
        &mut self,
        wm: &mut penrose::WindowManager<X>,
        id: penrose::Xid,
    ) -> penrose::Result<()> {
        let mouse_position = autopilot::mouse::location();

        let current_screen = (0..wm.n_screens())
            .into_iter()
            .map(|screen_index| wm.screen(&Selector::Index(screen_index)).unwrap())
            .find(|screen| {
                screen.contains(Point::new(mouse_position.x as u32, mouse_position.y as u32))
            })
            .unwrap();

        let current_workspace = current_screen.wix;

        let client = wm.client_mut(&Selector::WinId(id)).unwrap();
        client.set_workspace(current_workspace);

        Ok(())
    }
}

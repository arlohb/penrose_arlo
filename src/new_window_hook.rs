use penrose::{
    core::{data_types::Point, xconnection::XConn, Hook},
    Selector,
};

use lazy_static::lazy_static;

struct XData {
    pub conn: xcb::Connection,
    pub root: xcb::x::Window,
}

lazy_static! {
    static ref X_DATA: XData = {
        // Here screen does not relate to monitors, but the virtual screen made up of all monitors.
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();

        let root = screen.root();

        XData { conn, root }
    };
}

fn mouse_position() -> Point {
    // The xcb library is quite weird, so rust analyzer cannot infer the type of the returned value.
    let cookie: xcb::x::QueryPointerCookie = X_DATA.conn.send_request(&xcb::x::QueryPointer {
        window: X_DATA.root,
    });

    let reply: xcb::x::QueryPointerReply = X_DATA.conn.wait_for_reply(cookie).unwrap();

    Point::new(reply.root_x() as u32, reply.root_y() as u32)
}

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
        let mouse_position = mouse_position();

        let current_screen = (0..wm.n_screens())
            .into_iter()
            .map(|screen_index| wm.screen(&Selector::Index(screen_index)).unwrap())
            .find(|screen| screen.contains(mouse_position))
            .unwrap();

        let current_workspace = current_screen.wix;

        let client = wm.client_mut(&Selector::WinId(id)).unwrap();
        client.set_workspace(current_workspace);

        Ok(())
    }
}

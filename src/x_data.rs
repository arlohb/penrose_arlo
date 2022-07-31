use lazy_static::lazy_static;

pub struct XData {
    pub conn: xcb::Connection,
    pub root: xcb::x::Window,
}

lazy_static! {
    pub static ref X_DATA: XData = {
        // Here screen does not relate to monitors, but the virtual screen made up of all monitors.
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num.try_into().expect("X screen number was negative")).unwrap();

        let root = screen.root();

        XData { conn, root }
    };
}

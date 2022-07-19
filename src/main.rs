// The vec! macro often breaks autocomplete.
#![allow(clippy::vec_init_then_push)]

mod key_bindings;
pub use key_bindings::*;
mod new_window_hook;
pub use new_window_hook::*;
mod reactive_text;
pub use reactive_text::*;
mod setup;
pub use setup::*;

use penrose::{
    contrib::{extensions::Scratchpad, hooks::LayoutSymbolAsRootName},
    core::{
        config::Config,
        helpers::spawn,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Direction,
    },
    draw::*,
    xcb::{XcbConnection, XcbDraw, XcbDrawContext, XcbHooks},
    Selector,
};

use std::collections::HashMap;

const BAR_HEIGHT: usize = 22;

const FIRA: &str = "Fira Code";

pub struct Dracula;
impl Dracula {
    pub const BG: u32 = 0x282a36ff;
    pub const CURRENT_LINE: u32 = 0x44475aff;
    pub const SELECTION: u32 = 0x44475aff;
    pub const FG: u32 = 0xf8f8f2ff;
    pub const COMMENT: u32 = 0x6272a4ff;
    pub const CYAN: u32 = 0x8be9fdff;
    pub const GREEN: u32 = 0x50fa7bff;
    pub const ORANGE: u32 = 0xffb86cff;
    pub const PINK: u32 = 0xff79c6ff;
    pub const PURPLE: u32 = 0xbd93f9ff;
    pub const RED: u32 = 0xff5555ff;
    pub const YELLOW: u32 = 0xf1fa8cff;
}

fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    let mut clipboard = arboard::Clipboard::new().unwrap();

    let text_style = TextStyle {
        font: FIRA.to_string(),
        point_size: 12,
        fg: Dracula::FG.into(),
        bg: None,
        padding: (3., 3.),
    };

    let bar = StatusBar::<XcbDrawContext, XcbDraw, XcbConnection>::try_new(
        XcbDraw::new()?,
        penrose::draw::Position::Top,
        BAR_HEIGHT,
        Dracula::BG,
        &[FIRA],
        {
            let mut widgets: Vec<Box<dyn HookableWidget<XcbConnection>>> = Vec::new();

            widgets.push(ReactiveText::new(
                || Some("Left".to_string()),
                text_style.clone(),
                Align::Left,
                std::time::Duration::from_secs(5),
            ));
            widgets.push(ReactiveText::new(
                || Some("Center".to_string()),
                text_style.clone(),
                Align::Center,
                std::time::Duration::from_secs(5),
            ));

            let player_finder = mpris::PlayerFinder::new().unwrap();

            widgets.push(ReactiveText::new(
                move || {
                    let player = player_finder.find_active().ok()?;
                    let metadata = player.get_metadata().ok()?;

                    let title = match metadata.title() {
                        Some(title) if title.is_empty() => None,
                        Some(title) => Some(title.to_string()),
                        None => None,
                    };

                    let artists = match metadata.artists() {
                        Some(artists) if artists.is_empty() => None,
                        Some(artists) => Some(artists.join(", ")),
                        None => None,
                    };

                    match (title, artists) {
                        (Some(title), Some(artists)) => Some(format!("{} - {}", title, artists)),
                        (Some(title), None) => Some(title),
                        (None, Some(artists)) => Some(artists),
                        _ => None,
                    }
                },
                text_style,
                Align::Right,
                std::time::Duration::from_secs(5),
            ));

            widgets
        },
    )?;

    // Default number of clients in the main layout area
    let clients_in_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let main_to_min_ratio = 0.6;

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        // Windows with a matching WM_CLASS will always float
        .floating_classes(vec!["gnome-screenshot"])
        .focused_border(Color::new_from_hex(Dracula::PURPLE).as_rgb_hex_string())?
        .unfocused_border(Color::new_from_hex(Dracula::BG).as_rgb_hex_string())?
        // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
        // available to them, though they track modifications to n_main and ratio independently.
        .layouts(vec![
            Layout::new(
                "[side]",
                LayoutConf::default(),
                side_stack,
                clients_in_main,
                main_to_min_ratio,
            ),
            Layout::new(
                "[botm]",
                LayoutConf::default(),
                bottom_stack,
                clients_in_main,
                main_to_min_ratio,
            ),
        ])
        .bar_height(BAR_HEIGHT as u32)
        .gap_px(8)
        .build()
        .unwrap();

    let scratch_pad = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        LayoutSymbolAsRootName::new(),
        scratch_pad.get_hook(),
        Box::new(bar),
        NewWindowHook::new(),
    ];

    let mut keys = BetterKeyBindings::new();

    keys.add("super space", |_wm| {
        spawn("rofi -show run")?;
        Ok(())
    });

    keys.add("super ctrl escape", |wm| {
        wm.exit()?;
        Ok(())
    });

    keys.add("super T", |_wm| {
        spawn("kitty")?;
        Ok(())
    });

    keys.add("super Q", |wm| {
        wm.kill_client()?;
        Ok(())
    });

    keys.add("super E", |_wm| {
        spawn("thunar")?;
        Ok(())
    });

    keys.add("super slash", move |_wm| {
        (scratch_pad.toggle())(_wm)?;
        Ok(())
    });

    keys.add("super b", |_wm| {
        spawn("google-chrome")?;
        Ok(())
    });

    keys.add("super tab", |wm| {
        wm.drag_client(Direction::Forward)?;
        Ok(())
    });

    fn cycle_client_screen(
        wm: &mut WindowManager<XcbConnection>,
        direction: Direction,
    ) -> penrose::Result<()> {
        let focused_index = match wm.focused_client_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let current_screen = wm.active_screen_index();

        match direction {
            Direction::Forward => {
                if current_screen == wm.n_screens() - 1 {
                    wm.client_to_screen(&Selector::Index(0))?;
                } else {
                    wm.client_to_screen(&Selector::Index(current_screen + 1))?;
                }
            }
            Direction::Backward => {
                if current_screen == 0 {
                    wm.client_to_screen(&Selector::Index(wm.n_screens() - 1))?;
                } else {
                    wm.client_to_screen(&Selector::Index(current_screen - 1))?;
                }
            }
        }

        wm.focus_client(&Selector::WinId(focused_index))?;

        Ok(())
    }

    keys.add("super shift left", |wm| {
        cycle_client_screen(wm, Direction::Backward)?;
        Ok(())
    });

    keys.add("super shift right", |wm| {
        cycle_client_screen(wm, Direction::Forward)?;
        Ok(())
    });

    keys.add("super right", |wm| {
        wm.cycle_client(Direction::Forward)?;
        Ok(())
    });

    keys.add("super left", |wm| {
        wm.cycle_client(Direction::Backward)?;
        Ok(())
    });

    // Used `xev` to find the names for these

    keys.add("XF86AudioRaiseVolume", |_wm| {
        spawn("amixer set Master 5%+")?;
        Ok(())
    });

    keys.add("XF86AudioLowerVolume", |_wm| {
        spawn("amixer set Master 5%-")?;
        Ok(())
    });

    keys.add("XF86AudioMute", |_wm| {
        spawn("amixer set Master toggle")?;
        Ok(())
    });

    keys.add("super C", move |wm| {
        let client_id = match wm.focused_client_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let client = match wm.client(&Selector::WinId(client_id)) {
            Some(client) => client,
            None => return Ok(()),
        };

        let class = client.wm_class();

        let _ = clipboard.set_text(class.to_string());

        Ok(())
    });

    let mut wm = WindowManager::new(
        config,
        XcbConnection::new()?,
        hooks,
        penrose::logging_error_handler(),
    );
    wm.init()?;

    let mouse_bindings = HashMap::new();

    // mouse_bindings.insert(
    //     MouseEventKind::Motion,
    //     MouseState::new(MouseButton::)
    // );

    wm.grab_keys_and_run(keys.into_penrose_bindings(), mouse_bindings)?;

    Ok(())
}

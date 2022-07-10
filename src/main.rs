mod key_bindings;
pub use key_bindings::*;

fn setup_logger() {
    let home = std::env::var("HOME").unwrap();
    let log_file = format!("{home}/.penrose.log");

    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create(log_file).unwrap(),
    )
    .unwrap();
}

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
};

use std::collections::HashMap;

fn async_setup() {
    let _ = spawn("nitrogen --restore");
}

const BAR_HEIGHT: usize = 18;

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

    let bar = StatusBar::<XcbDrawContext, XcbDraw, XcbConnection>::try_new(
        XcbDraw::new()?,
        penrose::draw::Position::Top,
        BAR_HEIGHT,
        Dracula::BG,
        &[FIRA],
        // vec![
        // Box::new(widget::ActiveWindowName::new(
        //     &TextStyle {
        //         font: FIRA.to_string(),
        //         point_size: 12,
        //         fg: Dracula::FG.into(),
        //         bg: None,
        //         padding: (0., 0.),
        //     },
        //     100,
        //     false,
        //     false,
        // )),
        // Box::new(widget::Text::new(
        //     "Penrose Arlo",
        //     &TextStyle {
        //         font: FIRA.to_string(),
        //         point_size: 12,
        //         fg: Dracula::FG.into(),
        //         bg: None,
        //         padding: (-10., 2.),
        //     },
        //     true,
        //     true,
        // )),
        // Box::new(widget::Text::new(
        //     "BG",
        //     &TextStyle {
        //         font: FIRA.to_string(),
        //         point_size: 12,
        //         fg: Dracula::FG.into(),
        //         bg: Some(Dracula::BG.into()),
        //         padding: (-10., 2.),
        //     },
        //     false,
        //     false,
        // )),
        vec![
            ("BG", Dracula::BG),
            ("CURRENT_LINE", Dracula::CURRENT_LINE),
            ("SELECTION", Dracula::SELECTION),
            ("FG", Dracula::FG),
            ("COMMENT", Dracula::COMMENT),
            ("CYAN", Dracula::CYAN),
            ("GREEN", Dracula::GREEN),
            ("ORANGE", Dracula::ORANGE),
            ("PINK", Dracula::PINK),
            ("PURPLE", Dracula::PURPLE),
            ("RED", Dracula::RED),
            ("YELLOW", Dracula::YELLOW),
        ]
        .into_iter()
        .map(|(name, color)| {
            Box::new(widget::Text::new(
                name,
                &TextStyle {
                    font: FIRA.to_string(),
                    point_size: 12,
                    fg: Dracula::FG.into(),
                    bg: Some(color.into()),
                    padding: (12., 12.),
                },
                false,
                false,
            )) as Box<dyn HookableWidget<XcbConnection>>
        })
        .collect::<Vec<_>>(),
        // ],
    )?;

    // Default number of clients in the main layout area
    let n_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let ratio = 0.6;

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        // Windows with a matching WM_CLASS will always float
        .floating_classes(vec!["dmenu", "dunst", "polybar"])
        .focused_border(Color::new_from_hex(Dracula::PURPLE).as_rgb_hex_string())?
        .unfocused_border(Color::new_from_hex(Dracula::BG).as_rgb_hex_string())?
        // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
        // available to them, though they track modifications to n_main and ratio independently.
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        ])
        .bar_height(BAR_HEIGHT as u32)
        .gap_px(5)
        .build()
        .unwrap();

    let scratch_pad = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        LayoutSymbolAsRootName::new(),
        scratch_pad.get_hook(),
        Box::new(bar),
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

    // Only temporary untill I understand this more.
    // Spamming this a few times allows me to focus popups.
    keys.add("super tab", |wm| {
        wm.cycle_client(Direction::Forward)?;
        wm.drag_client(Direction::Forward)?;
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

    let mut wm = WindowManager::new(
        config,
        XcbConnection::new()?,
        hooks,
        penrose::logging_error_handler(),
    );
    wm.init()?;

    wm.grab_keys_and_run(keys.into_penrose_bindings(), HashMap::new())?;

    Ok(())
}

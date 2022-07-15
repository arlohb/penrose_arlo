mod command_listener;
mod key_bindings;
use command_listener::{CommandListener, Message};
pub use key_bindings::*;
mod new_window_hook;
pub use new_window_hook::*;

fn home() -> String {
    std::env::var("HOME").unwrap()
}

fn setup_logger() {
    let log_file = format!("{}/.penrose.log", home());

    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create(log_file).unwrap(),
    )
    .unwrap();

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{}", info);
    }));
}

use penrose::{
    contrib::{extensions::Scratchpad, hooks::LayoutSymbolAsRootName},
    core::{
        bindings::{MouseButton, MouseEventKind, MouseState},
        config::Config,
        helpers::spawn,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Direction,
    },
    draw::*,
    gen_mousebindings,
    xcb::{XcbConnection, XcbDraw, XcbDrawContext, XcbHooks},
    Selector,
};
use playerctl::PlayerCtl;

use std::{collections::HashMap, path::Path, process::Command};

fn async_setup() {
    let screens_script = format!("{}/penrose_arlo/screens.sh", home());

    if Path::new(&screens_script).exists() {
        let _: std::io::Result<()> = (|| {
            Command::new("bash").arg(screens_script).spawn()?.wait()?;
            Ok(())
        })();
    };

    let _ = spawn("nitrogen --restore");
}

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

pub enum Align {
    Left,
    Center,
    Right,
}

pub struct ReactiveText {
    text: Box<dyn FnMut() -> String>,
    text_style: TextStyle,
    align: Align,
    extent: Option<(f64, f64)>,
    last_updated: std::time::Instant,
    update_interval: std::time::Duration,
}

impl ReactiveText {
    pub fn new(
        text: impl FnMut() -> String + 'static,
        text_style: TextStyle,
        align: Align,
        update_interval: std::time::Duration,
    ) -> Box<Self> {
        Box::new(Self {
            text: Box::new(text),
            text_style,
            align,
            extent: None,
            last_updated: std::time::Instant::now(),
            update_interval,
        })
    }
}

impl<X> penrose::core::Hook<X> for ReactiveText where X: penrose::core::xconnection::XConn {}

impl Widget for ReactiveText {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        _screen: usize,
        _screen_has_focus: bool,
        _w: f64,
        _h: f64,
    ) -> Result<()> {
        ctx.font(&self.text_style.font, self.text_style.point_size)?;
        ctx.color(&self.text_style.fg);

        let text = (self.text)();

        // let extent = self.current_extent(ctx, h)?;

        // ctx.set_x_offset(match self.align {
        //     Align::Left => 0.,
        //     Align::Center => (w - extent.0) / 2.,
        //     Align::Right => w - extent.0,
        // });

        ctx.text(&text, 1., self.text_style.padding)?;

        self.last_updated = std::time::Instant::now();

        Ok(())
    }

    fn current_extent(&mut self, ctx: &mut dyn DrawContext, _h: f64) -> Result<(f64, f64)> {
        match self.extent {
            Some(extent) => Ok(extent),
            None => {
                let (l, r) = self.text_style.padding;
                ctx.font(&self.text_style.font, self.text_style.point_size)?;
                let (w, h) = ctx.text_extent(&(self.text)())?;
                let extent = (w + l + r + 0.1, h + 0.1);
                self.extent = Some(extent);
                Ok(extent)
            }
        }
    }

    fn require_draw(&self) -> bool {
        // self.last_updated.elapsed() > self.update_interval
        false
    }

    fn is_greedy(&self) -> bool {
        true
    }
}

fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    // let (command_sender, command_listener) = CommandListener::new();
    // command_sender.send(Message::new("Hello thread!!")).unwrap();

    // std::thread::spawn(move || {
    //     command_listener.listen();
    // });

    std::thread::spawn(|| loop {
        let child = std::process::Command::new("/bin/ls")
            // .arg("-c")
            // .arg("ls")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        match child.wait_with_output() {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).unwrap();
                tracing::info!("{}", stdout);
            }
            Err(e) => {
                tracing::error!("{:?}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

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
        vec![
            // ReactiveText::new(
            //     || PlayerCtl::metadata().title,
            //     text_style.clone(),
            //     Align::Right,
            //     std::time::Duration::from_secs(5),
            // ),
            // ReactiveText::new(
            //     || "Left".to_string(),
            //     text_style.clone(),
            //     Align::Left,
            //     std::time::Duration::from_secs(5),
            // ),
            // ReactiveText::new(
            //     || "Center".to_string(),
            //     text_style.clone(),
            //     Align::Center,
            //     std::time::Duration::from_secs(5),
            // ),
            // ReactiveText::new(
            //     || "Right".to_string(),
            //     text_style.clone(),
            //     Align::Right,
            //     std::time::Duration::from_secs(5),
            // ),
            ReactiveText::new(
                || {
                    // let child = std::process::Command::new("bash")
                    //     .arg("-c")
                    //     .arg("ls")
                    //     .stdout(std::process::Stdio::piped())
                    //     .spawn()
                    //     .unwrap();

                    // let stdout = child.wait_with_output().unwrap().stdout;
                    // let output = String::from_utf8(stdout).unwrap();

                    // tracing::info!("ls: {}", output);
                    // tracing::info!("{}", PlayerCtl::metadata().artist.is_empty());
                    "Hello".to_string()
                },
                text_style,
                Align::Left,
                std::time::Duration::from_secs(5),
            ),
        ],
    )?;

    // Default number of clients in the main layout area
    let clients_in_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let main_to_min_ratio = 0.6;

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

    // for i in 0..20 {
    //     let child = std::process::Command::new("bash")
    //         .arg("-c")
    //         .arg("ls")
    //         .stdout(std::process::Stdio::piped())
    //         .spawn()
    //         .unwrap();

    //     let stdout = child.wait_with_output().unwrap().stdout;
    //     let output = String::from_utf8(stdout).unwrap();
    //     tracing::info!("{i}: {output}");
    // }

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

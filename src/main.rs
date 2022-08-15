#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    // The vec! macro often breaks autocomplete.
    clippy::vec_init_then_push,
    // Is annoying.
    clippy::module_name_repetitions,
    // False(?) position when ignoring a result.
    clippy::let_underscore_drop,
    // Is annoying.
    clippy::cast_precision_loss,
    // Is annoying.
    clippy::cast_possible_truncation,
    // Is annoying.
    clippy::cast_possible_wrap,
    // Is annoying.
    clippy::cast_sign_loss,
    // Is annoying.
    clippy::cast_lossless,
)]

mod key_bindings;
pub use key_bindings::*;
mod new_window_hook;
pub use new_window_hook::*;
mod setup;
pub use setup::*;
mod wm_ext;
pub use wm_ext::*;
mod player;
pub use player::*;
mod colours;
pub use colours::*;
mod x_data;
pub use x_data::*;
mod window_switcher;
pub use window_switcher::*;
pub mod layouts;

use penrose::{
    contrib::{extensions::Scratchpad, hooks::LayoutSymbolAsRootName},
    core::{config::Config, helpers::spawn, manager::WindowManager, ring::Direction},
    draw::Color,
    xcb::{XcbConnection, XcbHooks},
    Selector,
};

use std::collections::HashMap;

#[allow(clippy::too_many_lines)]
fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    let mut clipboard = arboard::Clipboard::new().unwrap();

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        // Windows with a matching WM_CLASS will always float
        // java = minecraft?
        .floating_classes(vec!["gnome-screenshot", "java"])
        .focused_border(Color::new_from_hex(Dracula::PURPLE).as_rgb_hex_string())?
        .unfocused_border(Color::new_from_hex(Dracula::BG).as_rgb_hex_string())?
        // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
        // available to them, though they track modifications to n_main and ratio independently.
        .layouts(layouts::layouts())
        .show_bar(false)
        .gap_px(12)
        .build()
        .unwrap();

    let scratch_pad = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        LayoutSymbolAsRootName::new(),
        scratch_pad.get_hook(),
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

    keys.add("super slash", move |wm| {
        (scratch_pad.toggle())(wm)?;
        Ok(())
    });

    keys.add("super B", |_wm| {
        spawn("google-chrome")?;
        Ok(())
    });

    keys.add("super tab", |wm| {
        wm.drag_client(Direction::Forward)?;
        Ok(())
    });

    keys.add("super shift H", |wm| {
        wm.cycle_client_to_screen(Direction::Backward)?;
        Ok(())
    });

    keys.add("super shift L", |wm| {
        wm.cycle_client_to_screen(Direction::Forward)?;
        Ok(())
    });

    keys.add("super H", |wm| {
        wm.switch_focus_in_direction(SwitchDirection::Left)?;
        Ok(())
    });

    keys.add("super L", |wm| {
        wm.switch_focus_in_direction(SwitchDirection::Right)?;
        Ok(())
    });

    keys.add("super J", |wm| {
        wm.switch_focus_in_direction(SwitchDirection::Down)?;
        Ok(())
    });

    keys.add("super K", |wm| {
        wm.switch_focus_in_direction(SwitchDirection::Up)?;
        Ok(())
    });

    keys.add("super G", |wm| {
        wm.cycle_layout(Direction::Forward)?;
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

    keys.add("XF86AudioPlay", |_wm| {
        let _ = with_player(|player| player.play_pause().ok());
        Ok(())
    });

    keys.add("super P", |_wm| {
        let _ = with_player(|player| player.play_pause().ok());
        Ok(())
    });

    keys.add("XF86AudioStop", |_wm| {
        let _ = with_player(|player| player.stop().ok());
        Ok(())
    });

    keys.add("XF86AudioNext", |_wm| {
        let _ = with_player(|player| player.next().ok());
        Ok(())
    });

    keys.add("XF86AudioPrev", |_wm| {
        let _ = with_player(|player| player.previous().ok());
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

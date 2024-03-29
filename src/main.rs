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
    // Makes code less readable
    clippy::redundant_closure_for_method_calls,
    // I'm doing this on purpose so that func calls are done on start instead of on errors
    clippy::or_fun_call
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
    core::{
        config::Config, helpers::spawn, hooks::HooksVec, manager::WindowManager, ring::Direction,
    },
    draw::Color,
    xcb::XcbConnection,
    Selector,
};

use std::collections::HashMap;

#[allow(clippy::too_many_lines)]
fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    let mut clipboard = arboard::Clipboard::new().unwrap();

    let config = Config {
        workspaces: (1..=9).map(|i| i.to_string()).collect::<Vec<_>>(),
        floating_classes: vec!["gnome-screenshot".to_string(), "java".to_string()],
        focused_border: Color::new_from_hex(Dracula::PURPLE),
        unfocused_border: Color::new_from_hex(Dracula::BG),
        layouts: layouts::layouts(),
        show_bar: false,
        gap_px: 12,
        ..Default::default()
    }
    .validate()?;

    let scratch_pad = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: HooksVec<_> = vec![
        LayoutSymbolAsRootName::new(),
        scratch_pad.get_hook(),
        NewWindowHook::new(),
    ];

    let mut keys = BetterKeyBindings::new();

    // Program runners
    keys.add("meta T", |_wm| spawn("kitty"));
    keys.add("meta E", |_wm| spawn("thunar"));
    keys.add("meta B", |_wm| spawn("vivaldi-stable"));
    keys.add("meta shift B", |_wm| spawn("vivaldi-stable --incognito"));

    // Other runners
    keys.add("meta space", |_wm| spawn("rofi -modi drun -show drun"));
    keys.add("meta slash", move |wm| (scratch_pad.toggle())(wm));

    // Penrose commands
    keys.add("meta ctrl escape", |wm| wm.exit());
    keys.add("meta G", |wm| wm.cycle_layout(Direction::Forward));
    keys.add("meta C", move |wm| {
        clipboard
            .set_text(
                wm.client(&Selector::Focused)
                    .ok_or(penrose::PenroseError::Raw("No focused client".to_string()))?
                    .wm_class()
                    .to_string(),
            )
            .map_err(|_| penrose::PenroseError::Raw("Failed to save to clipboard".to_string()))
    });

    // Client management
    keys.add("meta Q", |wm| wm.kill_client());

    // Stuff in all 4 directions
    for (key_options, direction) in [
        (["H", "left"], SwitchDirection::Left),
        (["L", "right"], SwitchDirection::Right),
        (["K", "up"], SwitchDirection::Up),
        (["J", "down"], SwitchDirection::Down),
    ] {
        for key in key_options {
            // Switching between clients
            keys.add(format!("meta {key}"), move |wm| {
                wm.switch_focus_in_direction(direction)
            });
        }
    }

    keys.add("meta tab", |wm| wm.drag_client(Direction::Forward));

    // Stuff in only 2 directions
    for (key_options, direction) in [
        (["H", "left"], Direction::Backward),
        (["L", "right"], Direction::Forward),
    ] {
        for key in key_options {
            // Move client to screen
            keys.add(format!("meta shift {key}"), move |wm| {
                wm.cycle_client_to_screen(direction)
            });

            // Move to workspace
            keys.add(format!("meta ctrl {key}"), move |wm| {
                wm.cycle_workspace(direction)
            });
        }
    }

    // Workspace management
    for i in config.ws_range() {
        // Switch to workspace i
        keys.add(format!("meta {i}"), move |wm| {
            wm.focus_workspace(&Selector::Index(i - 1))?;
            Ok(())
        });

        // Move client to workspace i
        keys.add(format!("meta shift {i}"), move |wm| {
            wm.client_to_workspace(&Selector::Index(i - 1))?;
            Ok(())
        });
    }

    // Used `xev` to find the names for these

    // Volume control
    keys.add("XF86AudioRaiseVolume", |_wm| spawn("amixer set Master 5%+"));
    keys.add("XF86AudioLowerVolume", |_wm| spawn("amixer set Master 5%-"));
    keys.add("XF86AudioMute", |_wm| spawn("amixer set Master toggle"));

    // Playback control
    keys.add("XF86AudioPlay", |_wm| {
        with_player(|player| player.play_pause().ok()).ok_or(penrose::PenroseError::Raw(
            "Audio control failed".to_string(),
        ))
    });
    keys.add("meta P", |_wm| {
        with_player(|player| player.play_pause().ok()).ok_or(penrose::PenroseError::Raw(
            "Audio control failed".to_string(),
        ))
    });
    keys.add("XF86AudioStop", |_wm| {
        with_player(|player| player.stop().ok()).ok_or(penrose::PenroseError::Raw(
            "Audio control failed".to_string(),
        ))
    });
    keys.add("XF86AudioNext", |_wm| {
        with_player(|player| player.next().ok()).ok_or(penrose::PenroseError::Raw(
            "Audio control failed".to_string(),
        ))
    });
    keys.add("XF86AudioPrev", |_wm| {
        with_player(|player| player.previous().ok()).ok_or(penrose::PenroseError::Raw(
            "Audio control failed".to_string(),
        ))
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

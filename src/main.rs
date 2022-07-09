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
    },
    xcb::{XcbConnection, XcbHooks},
};

use std::{collections::HashMap, process::Command};

fn async_setup() {
    let _ = Command::new("nitrogen").arg("--restore").spawn();
}

fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    // Default number of clients in the main layout area
    let n_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let ratio = 0.6;

    let mut config_builder = Config::default().builder();
    let config = config_builder
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        // Windows with a matching WM_CLASS will always float
        .floating_classes(vec!["dmenu", "dunst", "polybar"])
        .focused_border("#cc241d")?
        .unfocused_border("#3c3836")?
        // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
        // available to them, though they track modifications to n_main and ratio independently.
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        ])
        .build()
        .unwrap();

    let scratch_pad = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: XcbHooks = vec![LayoutSymbolAsRootName::new(), scratch_pad.get_hook()];

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

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
        hooks::Hook,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Selector,
        xconnection::{XConn, Xid},
    },
    xcb::{XcbConnection, XcbHooks},
};

use std::{collections::HashMap, process::Command};
use tracing::info;

// An example of a simple custom hook. In this case we are creating a NewClientHook which will
// be run each time a new client program is spawned.
struct MyClientHook {}
impl<X: XConn> Hook<X> for MyClientHook {
    fn new_client(&mut self, wm: &mut WindowManager<X>, id: Xid) -> penrose::Result<()> {
        let c = wm.client(&Selector::WinId(id)).unwrap();
        info!("new client with WM_CLASS='{}'", c.wm_class());
        Ok(())
    }
}

fn async_setup() {
    let _ = Command::new("nitrogen").arg("--restore").spawn();
}

fn main() -> penrose::Result<()> {
    setup_logger();
    std::thread::spawn(async_setup);

    // Created at startup. See keybindings below for how to access them
    let mut config_builder = Config::default().builder();
    config_builder
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        // Windows with a matching WM_CLASS will always float
        .floating_classes(vec!["dmenu", "dunst", "polybar"])
        // Client border colors are set based on X focus
        .focused_border("#cc241d")?
        .unfocused_border("#3c3836")?;

    // Default number of clients in the main layout area
    let n_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let ratio = 0.6;

    // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
    // available to them, though they track modifications to n_main and ratio independently.
    config_builder.layouts(vec![
        Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
    ]);

    // Now build and validate the config
    let config = config_builder.build().unwrap();

    /* hooks
     *
     * penrose provides several hook points where you can run your own code as part of
     * WindowManager methods. This allows you to trigger custom code without having to use a key
     * binding to do so. See the hooks module in the docs for details of what hooks are avaliable
     * and when/how they will be called. Note that each class of hook will be called in the order
     * that they are defined. Hooks may maintain their own internal state which they can use to
     * modify their behaviour if desired.
     */

    // Scratchpad is an extension: it makes use of the same Hook points as the examples below but
    // additionally provides a 'toggle' method that can be bound to a key combination in order to
    // trigger the bound scratchpad client.
    let sp = Scratchpad::new("mousepad", 0.8, 0.8);

    let hooks: XcbHooks = vec![
        Box::new(MyClientHook {}),
        // Using a simple contrib hook that takes no config. By convention, contrib hooks have a 'new'
        // method that returns a boxed instance of the hook with any configuration performed so that it
        // is ready to push onto the corresponding *_hooks vec.
        LayoutSymbolAsRootName::new(),
        sp.get_hook(),
    ];

    /* The gen_keybindings macro parses user friendly key binding definitions into X keycodes and
     * modifier masks. It uses the 'xmodmap' program to determine your current keymap and create
     * the bindings dynamically on startup. If this feels a little too magical then you can
     * alternatively construct a  HashMap<KeyCode, FireAndForget> manually with your chosen
     * keybindings (see helpers.rs and data_types.rs for details).
     * FireAndForget functions do not need to make use of the mutable WindowManager reference they
     * are passed if it is not required: the run_external macro ignores the WindowManager itself
     * and instead spawns a new child process.
     */

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
        (sp.toggle())(_wm)?;
        Ok(())
    });

    // let key_bindings = gen_keybindings! {
    //     // client management
    //     "M-j" => run_internal!(cycle_client, Forward);
    //     "M-k" => run_internal!(cycle_client, Backward);
    //     "M-S-j" => run_internal!(drag_client, Forward);
    //     "M-S-k" => run_internal!(drag_client, Backward);
    //     "M-S-q" => run_internal!(kill_client);
    //     "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);

    //     // workspace management
    //     "M-Tab" => run_internal!(toggle_workspace);
    //     "M-bracketright" => run_internal!(cycle_screen, Forward);
    //     "M-bracketleft" => run_internal!(cycle_screen, Backward);
    //     "M-S-bracketright" => run_internal!(drag_workspace, Forward);
    //     "M-S-bracketleft" => run_internal!(drag_workspace, Backward);

    //     // Layout management
    //     "M-grave" => run_internal!(cycle_layout, Forward);
    //     "M-S-grave" => run_internal!(cycle_layout, Backward);
    //     "M-A-Up" => run_internal!(update_max_main, More);
    //     "M-A-Down" => run_internal!(update_max_main, Less);
    //     "M-A-Right" => run_internal!(update_main_ratio, More);
    //     "M-A-Left" => run_internal!(update_main_ratio, Less);

    //     "M-A-s" => run_internal!(detect_screens);

    //     // Each keybinding here will be templated in with the workspace index of each workspace,
    //     // allowing for common workspace actions to be bound at once.
    //     map: { "1", "2", "3", "4", "5", "6", "7", "8", "9" } to index_selectors(9) => {
    //         "M-{}" => focus_workspace (REF);
    //         "M-S-{}" => client_to_workspace (REF);
    //     };
    // };

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

use std::{path::Path, process::Command};

use penrose::common::helpers::spawn;

#[must_use]
pub fn home() -> String {
    std::env::var("HOME").expect("HOME is not set")
}

pub fn setup_logger() {
    let log_file = format!("{}/.penrose.log", home());

    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create(log_file).expect("Failed to create log file"),
    )
    .expect("Failed to initialize logger");

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("{}", info);
    }));
}

pub fn async_setup() {
    let screens_script = format!("{}/penrose_arlo/screens.sh", home());

    if Path::new(&screens_script).exists() {
        let _ = (|| -> std::io::Result<()> {
            Command::new("bash").arg(screens_script).spawn()?.wait()?;
            Ok(())
        })();
    };

    let _ = spawn("nitrogen --restore");

    let _ = spawn("picom --experimental-backends");

    let _ = spawn("setxkbmap -option caps:escape");
}

use std::{path::Path, process::Command};

use penrose::core::helpers::spawn;

pub fn home() -> String {
    std::env::var("HOME").unwrap()
}

pub fn setup_logger() {
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

pub fn async_setup() {
    let screens_script = format!("{}/penrose_arlo/screens.sh", home());

    if Path::new(&screens_script).exists() {
        let _: std::io::Result<()> = (|| {
            Command::new("bash").arg(screens_script).spawn()?.wait()?;
            Ok(())
        })();
    };

    let _ = spawn("nitrogen --restore");

    let _ = spawn("picom --experimental-backends");
}

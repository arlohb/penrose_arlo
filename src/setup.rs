use std::process::Command;

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

/// Run a given bash script
///
/// # Errors
///
/// Returns an error if the script fails to execute
/// or does not exist
pub fn run_script(path: impl AsRef<std::ffi::OsStr>, wait: bool) -> std::io::Result<()> {
    if std::path::Path::new(&path).exists() {
        let mut child = Command::new("bash").arg(path).spawn()?;

        if wait {
            child.wait()?;
        }

        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))
    }
}

pub fn async_setup() {
    let _ = run_script(format!("{}/penrose_arlo/screens.sh", home()), true);

    let _ = spawn("nitrogen --restore");

    let _ = spawn("picom --experimental-backends");

    let _ = spawn("setxkbmap -option caps:escape");
}

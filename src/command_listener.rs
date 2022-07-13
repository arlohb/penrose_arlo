use std::sync::mpsc;

pub struct Message(String);

impl Message {
    pub fn new(text: &str) -> Self {
        Message(text.to_string())
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

pub struct CommandListener {
    rx: mpsc::Receiver<Message>,
}

impl CommandListener {
    pub fn new() -> (mpsc::Sender<Message>, Self) {
        let (tx, rx) = mpsc::channel();
        (tx, Self { rx })
    }

    pub fn listen(&self) -> ! {
        loop {
            let message = self.rx.recv().unwrap();
            tracing::info!("{}", message.text());

            let child = std::process::Command::new("bash")
                .arg("-c")
                .arg("ls")
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            let stdout = child.wait_with_output().unwrap().stdout;
            let output = String::from_utf8(stdout).unwrap();
            tracing::info!("{}", output);
        }
    }
}

# How to use

```bash
sudo apt install git
git clone https://github.com/arlohb/penrose_arlo.git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo build --release
```

Change Exec and TryExec in penrose_arlo.desktop to {{PROJECT_DIR}}/target/release/penrose_arlo

Copy penrose_arlo.desktop to /usr/share/xsessions.
This will mean it is picked up by the login manager you're using.

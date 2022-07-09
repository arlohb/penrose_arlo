# How to setup debian

```bash

sudo -i
rmmod pcspkr ; echo "blacklist pcspkr" >>/etc/modprobe.d/blacklist.conf
exit

sudo apt install git gh micro
gh auth login
git clone https://github.com/arlohb/penrose_arlo.git
cd penrose_arlo

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

sudo apt install build-essential pkg-config libgtk-3-dev libxcb-randr0-dev

cargo build --release

sudo apt install rofi nitrogen gnome-backgrounds
nitrogen /usr/share/backgrounds/ --save

```

Change Exec and TryExec in penrose_arlo.desktop to {{PROJECT_DIR}}/target/release/penrose_arlo

Copy penrose_arlo.desktop to /usr/share/xsessions.
This will mean it is picked up by the login manager you're using.

You can now login to penrose.

```bash

nitrogen /usr/share/wallpapers/ --save

```

#!/bin/bash

# Install dependencies
sudo apt install build-essential pkg-config libgtk-3-dev libxcb-randr0-dev playerctl

# Build
cargo build --release

# Install
sudo cp penrose_arlo.desktop /usr/share/xsessions/

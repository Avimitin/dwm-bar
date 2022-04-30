# dwm-bar

This is a simple program for showing information on the dwm bar.

***ALERT!!!!: This is a WORK IN PROGRESS project, use at your own risk.*** 

## Features

Currently, it supports:

* Date and Time
* Battery
* Bluetooth Headset Battery
* Sound Volume
* Song Information

## Prerequisite

- DBus (For song information and bluetooth headset battery)
- PulseAudio (For volume)

## Build

```bash
# Default
cargo install --path .

# Enable headset battery
cargo install --path . --features headset-battery
```

## Usage

```bash
~/.cargo/bin/dwm-bar &
```

## Todo

- [x] tokio async io
- [x] native PulseAudio control
- [x] native battery information
- [ ] easy configuration
- [ ] API port for new module
- [ ] Flexible code design

## Gallery

![image](./images/image.png)

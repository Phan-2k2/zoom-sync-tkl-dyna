# zoom-sync

Cross-platform utility to sync Zoom65 v3 screen modules.

## Features

|                     | zoom-sync              | MeletrixID / WuqueID            |
| ------------------- | ---------------------- | ------------------------------- |
| Supported platforms | Cross-platform         | Windows, OSX                    |
| FOSS ?              | FOSS. Always.          | Free, but not open sourced      |
| Languages           | English                | Chinese or English              |
| Weather api         | [open-meteo](https://open-meteo.com) | Unknown centralized service |
| Geolocation api     | [ipinfo](https://ipinfo.io) or manual | Bundled into weather api |
| VPN workaround      | Manual geo coordinates | Not supported                     |
| Temperature units   | °C or °F               | °C only                         |
| Time sync           | Supported              | Supported                       |
| CPU temperature     | Supported              | Supported                       |
| GPU temperature     | Nvidia only            | Supported                       |
| Download rate       | Manual only            | Supported                       |
| Manually set data   | Supported              | Not supported                   |
| Image/gif upload    | Supported              | Not supported (use web driver)  |
| Reactive image/gif  | Simulated              | Not supported                   |
| Future-proof        | Will always work       | Overflow errors after year 2255 |

## Third Party Services

The following free third-party services are used to fetch some information:

- Weather forcasting: [open-meteo](https://open-meteo.com)
- Geolocation (optional for automatic weather coordinates): [ipinfo.io](https://ipinfo.io)

## Installation

Requirements:

- libudev (linux, included with systemd)
- openssl
- rust/rustup

### Source

```bash
git clone https://github.com/ozwaldorf/zoom-sync && cd zoom-sync
cargo install --path .
```

### Crates.io

```bash
cargo install zoom-sync
```

### Nix

> Note: On nixos, you must use the flake for nvidia gpu temp to work

```bash
nix run github:ozwaldorf/zoom-sync
```

## Usage

### CLI

Detailed command line documentation can be found in [docs/README.md](./docs/README.md).

### Running on startup

#### Linux / systemd

A systemd service can be easily setup that will manage running zoom-sync.
An example can be found at [docs/zoom-sync.service](./docs/zoom-sync.service).

```bash
# edit configuration arguments in ExecStart
vim docs/zoom-sync.service

# copy to system services
sudo cp docs/zoom-sync.service /etc/systemd/system

# enable and start the servive
sudo systemctl enable --now zoom-sync.service
```

#### Windows

> TODO

#### OSX

> TODO

### Simple examples

```bash
# Only update time and weather, and set the screen to weather on connect:
zoom-sync --no-system --screen weather

# Only update time and system info, and set the screen to cpu temp on connect:
zoom-sync --no-weather --screen cpu

# Use hardcoded coordinates for fetching weather
zoom-sync --coords 27.1127 109.3497

# use a gif as both static and animated image, run with reactive mode enabled and no other data
zoom-sync set image my-anim.gif
zoom-sync set gif my-anim.gif
zoom-sync --reactive --no-system --no-weather

# clear image and gif back to the chrome dino and nyancat
zoom-sync set image clear
zoom-sync set gif clear

# set time
zoom-sync set time

# set weather manually
zoom-sync set weather -w 0 10 20 5
```

## Feature Checklist

- [x] Reverse engineer updating each value
  - [x] Time
  - [x] Weather (current, min, max)
  - [x] CPU/GPU temp
  - [x] Download rate
  - [x] Screen up/down/switch
  - [x] GIF image
  - [x] Static image
- [x] Fetch current weather report
- [x] Fetch CPU temp
- [x] Fetch GPU temp
  - [x] Nvidia
  - [ ] AMD
- [ ] Monitor download rate
- [x] Poll and reconnect to keyboard
- [x] CLI arguments
- [x] Update intervals for each value
- [x] Simulate reactive gif mode (linux)
- [ ] System tray menu
- [ ] Package releases
  - [x] Crates.io
  - [ ] Nixpkgs
  - [ ] Windows
  - [ ] OSX

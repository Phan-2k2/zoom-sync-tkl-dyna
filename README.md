# NOTICE:
This project is a fork of ozwaldorf's zoom-sync, which you can find [here](https://github.com/ozboar/zoom-sync), an attempt to get some functionality working w/Meletrix's Zoom TKL DYNA. This additional support has since been rolled into their project and will therefore this specific fork will no longer see additional updates or upgrades. 

This fork has support for the following on the DYNA:
- Update Time
- Update Weather
- Update Temps, Fan Speed, with manual support for CPU/downloads.

In addition, there are a few changes:
- Instead of running one application that connects to all supported keybooards, I split out the executables to generate separately of one another. In this example, we generate an executable for zoom65v3, and zoomtkldyna. 
- An attempt to make the execution synchronous for future support for some system trays, but I got busy and never got around to it.

I'll keep this repo open for future reference, but if I want to keep working on something like this, I'll probably reiterate on the newly added features that ozwaldorf has - they've done a great job reverse engineering stuff, especially w/o physically having the keyboard.

# zoom-sync

Cross-platform utility to sync Zoom65 v3 screen modules.

## Features

> Note: All features marked "simulated" are not supported by the screen firmware natively, but rather achieved by the zoom-sync process.

|                     | zoom-sync              | MeletrixID / WuqueID            |
| ------------------- | ---------------------- | ------------------------------- |
| Supported platforms | Cross-platform         | Windows, OSX                    |
| FOSS ?              | FOSS. Always.          | Free, but not open sourced      |
| Languages           | English                | Chinese or English              |
| Weather API         | [open-meteo](https://open-meteo.com) | Unknown centralized service |
| Geolocation API     | [ipinfo](https://ipinfo.io) or manual | Bundled into weather api |
| VPN workaround      | Manual geo coordinates | Not supported                   |
| Temperature units   | °C or simulated °F     | °C only                         |
| Time sync           | Supported              | Supported                       |
| 12hr time           | Simulated              | Not supported                   |
| CPU temperature     | Supported              | Supported                       |
| GPU temperature     | Nvidia only            | Supported                       |
| Download rate       | Manual only            | Supported                       |
| Manually set data   | Supported              | Not supported                   |
| Image/gif upload    | Supported w/ custom bg | Not supported (use web driver)  |
| Reactive image/gif  | Simulated              | Not supported                   |
| Future-proof        | Will always work       | Overflow errors after year 2255 |

## Third Party Services

The following free third-party services are used to fetch some information:

- Weather forcasting: [open-meteo](https://open-meteo.com)
- Geolocation (optional for automatic weather coordinates): [ipinfo.io](https://ipinfo.io)

## Installation

> See the [latest release notes](https://github.com/ozboar/zoom-sync/releases/latest) for pre-built windows and linux binaries

Build requirements:

- rust/rustup
- openssl
- libudev (linux only, included with systemd)

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

A systemd service can be easily setup that will manage running zoom-sync on boot.
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

1. Locate the zoom-sync.exe file depending on the installation and open in the file manager
   - From source or crates.io: Press Windows + R (Run) and enter `%userprofile%\.cargo\bin`
2. Create a new shortcut to your zoom-sync.exe (right click -> create shortcut)
3. Edit shortcut (right click -> properties) and add any configuration arguments to the `target` after `zoom-sync.exe`
4. Press Windows + R (Run) and type `shell:startup`
5. Move the newly created shortcut to the opened startup applications folder to have zoom-sync run automatically on boot

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

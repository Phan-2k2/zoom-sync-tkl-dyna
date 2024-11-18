# zoom-sync

Cross-platform utility to sync Zoom65 v3 screen modules.

## Comparison

|                     | zoom-sync        | MeletrixID                      |
| ------------------- | ---------------- | ------------------------------- |
| Supported platforms | cross-platform   | Windows, OSX                    |
| FOSS ?              | FOSS. Always.    | Free, but not open sourced      |
| Languages           | English          | Chinese or english              |
| Weather api         | [open-meteo](https://open-meteo.com) | Unknown centralized service |
| Geolocation api     | [ipinfo](https://ipinfo.io) or manual | Bundled into weather api |
| VPN workaround      | With manual geo  | Only uses vpn's ip for location |
| Temperature units   | °C or °F         | °C only                         |
| Time sync           | Supported        | Supported                       |
| CPU temperature     | Supported        | Supported                       |
| GPU temperature     | Nvidia           | Supported ?                     |
| Download rate       | WIP              | Supported                       |
| Manual data         | Supported        | Not supported                   |
| Single update mode  | Supported        | Not supported                   |
| Future-proof        | Will always work | Overflow errors after year 2255 |

## Third Party Services

The following free third-party services are used to fetch some information:

- Weather forcasting: [open-meteo](https://open-meteo.com)
- Geolocation (optional for automatic weather coordinates): [ipinfo.io](https://ipinfo.io)

## Installation

### Source

Requirements:

- libudev (linux, included with systemd)
- openssl

```
git clone https://github.com/ozwaldorf/zoom-sync && cd zoom-sync
cargo install --path .
```

### Nix

> Note: On nixos, you must use the flake for nvidia gpu temp to work

```
nix run github:ozwaldorf/zoom-sync
```

## Usage

```
Usage: zoom-sync [-r=SECS] [-f] [-g=ID] [-t=LABEL] [-c=<LAT,LONG>]

Available options:
    -r, --refresh=SECS       Refresh data every given number of seconds
                             [default: 30]
    -f, --farenheit          Use farenheit for all temperatures. May cause
                             clamping for anything greater than 99F
                             [default: false]
    -g, --gpu=ID             Use a specific gpu device id
                             [default: 0]
    -t, --temp=LABEL         Search for a specific cpu temp component
                             [default: coretemp Package]
    -c, --coords=<LAT,LONG>  Optional coordinates to use for open-meteo weather
                             forcasting. If unset, falls back to ipinfo.com for
                             location.
    -h, --help               Prints help information
    -V, --version            Prints version information
```

## Feature Checklist

- [x] Reverse engineer updating each value
  - [x] Time
  - [x] Weather (current, min, max)
  - [x] CPU/GPU temp
  - [x] Download rate
- [x] Fetch current weather report
- [x] Fetch CPU temp
- [x] Fetch Nvidia GPU temp
- [ ] Fetch AMD GPU temp
- [ ] Monitor download rate
- [x] CLI arguments
- [ ] System tray menu
  - [ ] Poll and autodetect new keyboards
  - [ ] Update intervals for each value
- [ ] Package releases
  - [ ] Crates.io
  - [ ] Nixpkgs
  - [ ] Windows
  - [ ] OSX

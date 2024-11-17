# zoom-sync

Cross-platform utility to sync Zoom65 v3 screen modules.

## Third Party Services

The following free third-party services are used to fetch some information:

- Weather forcasting: [open-meteo](https://open-meteo.com)
- Geolocation (optional for automatic weather coordinates): [ipinfo.io](https://ipinfo.io)

## Installation

#### Requirements

- libudev (linux)
- openssl

### Source

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
- [x] Fetch GPU temp (nvidia only)
- [x] Fetch CPU temp
- [ ] Monitor download rate
- [x] CLI arguments
- [ ] System tray menu
  - [ ] Poll and autodetect new keyboards
  - [ ] Update intervals for each value
- [ ] Package release for version 1.0
  - [ ] Crates.io
  - [ ] Nixpkgs
  - [ ] Windows

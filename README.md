# zoom-sync

Cross-platform, open source utility to sync time, weather, and system info on Zoom65 v3 screen modules.

### Third Party Services

The following free third-party services are used to fetch some information:

- Weather forcasting: [open-meteo](https://open-meteo.com)
- Geolocation (for weather): [ipinfo.io](https://ipinfo.io)

## Usage

> Note: On nixos, you must use the flake for nvidia gpu temp to work

```
nix run
# or
cargo run
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
- [x] CLI options
- [ ] System tray menu
  - [ ] Poll and autodetect new keyboards
  - [ ] Update intervals for each value
- [ ] Package release for version 1.0
  - [ ] Crates.io
  - [ ] Nixpkgs
  - [ ] Windows

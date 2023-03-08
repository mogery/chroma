# chroma
One copy of Electron to rule them all.

chroma keeps a central, up-to-date version of Electron, and makes all your installed Electron apps use it, in order to save disk space.

## State of the Project

Supported platforms:
 * [X] Linux
    * [X] regular installs
    * [ ] flatpak (WIP: sandboxing issues)
    * [ ] AppImages
 * [ ] Windows
 * [ ] macOS

## Installing

Rust and cargo are required. Install them with [rustup](https://rustup.rs/).

Installation is a little rough and unintuitive right now, due to the lack of code for fetching the latest Electron. It'll get easier in the future.

 * as root: `mkdir /var/lib/chroma/`
 * grab the [latest electron ZIP](https://github.com/electron/electron/releases/latest) for your platform
 * as root: unzip into `/var/lib/chroma/electron`
 * as self: `cargo install --git https://github.com/mogery/chroma`
 
## Usage

### Regular installs

For example, if you want to chromafy Slack:
```bash
chroma raw $(where slack)
```

### Flatpak

**WARNING: Flatpak support does not work right now: this will mangle your app.**

For example, if you want to chromafy Slack:
```bash
chroma flatpak com.slack.Slack
```

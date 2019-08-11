glrnvim
=======
[![Build Status](https://travis-ci.com/beeender/glrnvim.svg?branch=master)](https://travis-ci.com/beeender/glrnvim)

A really fast & stable neovim GUI could be accelated by GPU.

See the speed from the screenshot:
![screenshot](screenshot/very_fast.gif)

## About

glrnvim combines Open**GL** (possibly), **R**ust and **N**eo**VIM** together, to make the fastest, simplest neovim GUI.

The above things are not totally lie. The intention of this project was that I couldn't find a standalone neovim GUI fast and stable like the old gvim. All the existing neovim GUIs are either not fast enough (what do you expect from Electron?) or not stable enough. I have been using [neovim-gnome-terminal-wrapper](https://github.com/fmoralesc/neovim-gnome-terminal-wrapper) for a long time and it is much better than any other fancy GUIs. The only thing is, it only supports gnome-terminal.

glrnvim wraps nvim with your favourite terminal into a standalone, non-fancy but daily-usable neovim GUI.

## Requisites

* [alacritty](https://github.com/jwilm/alacritty)/[kitty](https://github.com/kovidgoyal/kitty)/[rxvt-unicode](http://software.schmorp.de/pkg/rxvt-unicode.html)
* [neovim](https://neovim.io)

## Installation

### Arch Linux

Install `glrnvim` from the AUR.

### Debian/Ubuntu

- Install [cargo-deb](https://github.com/mmstick/cargo-deb).

```
cargo install cargo-deb
```

- Build and install the `deb` package system-wide.

```
cargo deb --install
```

## Build

### Linux & macOS

```sh
cargo build
```

## Windows

Not difficult, just try.

## Configuration

Modify [example config](https://github.com/beeender/glrnvim/blob/master/glrnvim.yml) and copy it to your `XDG_CONFIG_HOME` directory.

- For Linux: `$HOME/.config/glrnvim.yml`
- For MacOS: `$HOME/Library/Preferences/glrnvim.yml`

## Tips

### Set `glrnvim` as the git editor for commit message

```sh
git config --global core.editor "glrnvim --nofork"
```

## Known Issues:

_Color scheme doesn't work well with urxvt backend._

glrnvim uses `set termguicolors` to achieve an easy and better color scheme support. However, that requires the terminal to support true colors. Urxvt never has an official release to support it. Although the true color patch has been merged many years ago. If you are using Arch, just install [rxvt-unicode-cvs](https://aur.archlinux.org/packages/rxvt-unicode-cvs) from aur.

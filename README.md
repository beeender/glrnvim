glrnvim
=======

A GPU-accelerated neovim GUI.


glrnvim is the fastest neovim gui in existence. Using the GPU for rendering enables optimizations that simply aren't possible without it.

See the speed from the screenshot:
![screenshot](screenshot/very_fast.gif)

## About

glrnvim combines Open**GL**, **R**ust and **N**eo**VIM** together, to make the fastest, simplist neovim GUI.

## Requisites

* [alacritty](https://github.com/jwilm/alacritty)
* [neovim](https://neovim.io)

## Installation

### Arch Linux

Install `glrnvim` from the AUR.

## Build

### Linux & macOS

```sh
cargo build
```

## Windows

Not difficult, just try.

## Configuration

glrnvim comes with a very flexible configuration ability, just copy [glrnvim.yml](https://github.com/beeender/glrnvim/blob/master/glrnvim.yml) to `$HOME/.config` (for Linux) or `$HOME/Library/Preferences` (for macOS) and modify it.

Please refer to [this](https://github.com/jwilm/alacritty/wiki/Changing-the-default-font) for more information about setting font.

## Roadmap

* Make a rendering benchmark vim plugin to prove this is the fastest neovim GUI.

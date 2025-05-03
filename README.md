glrnvim
<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-10-orange.svg?style=flat-square)](#contributors-)
<!-- ALL-CONTRIBUTORS-BADGE:END -->

=======

A really fast & stable neovim GUI could be accelated by GPU.

See the speed from the screenshot:
![screenshot](screenshot/very_fast.gif)

## About

glrnvim combines Open**GL** (possibly), **R**ust and **N**eo**VIM** together, to make the fastest, simplest neovim GUI.

The above things are not totally lie. The intention of this project was that I couldn't find a standalone neovim GUI fast and stable like the old gvim. All the existing neovim GUIs are either not fast enough (what do you expect from Electron?) or not stable enough. I have been using [neovim-gnome-terminal-wrapper](https://github.com/fmoralesc/neovim-gnome-terminal-wrapper) for a long time and it is much better than any other fancy GUIs. The only thing is, it doesn't support other terminals than gnome-terminal.

glrnvim wraps nvim with your favourite terminal into a standalone, non-fancy but daily-usable neovim GUI.

## Requisites

* [alacritty](https://github.com/jwilm/alacritty)/[kitty](https://github.com/kovidgoyal/kitty)/[rxvt-unicode](http://software.schmorp.de/pkg/rxvt-unicode.html)/[wezterm](https://wezfurlong.org/wezterm/)/[foot](https://codeberg.org/dnkl/foot)
* [neovim](https://neovim.io)

## Installation

### Arch Linux

Install `glrnvim` from the AUR.

### Debian/Ubuntu

- Install [cargo-deb](https://github.com/mmstick/cargo-deb).

```
cargo install cargo-deb
```

- Clone the project. Then build and install the `deb` package system-wide by running the following command from the project's root directory.

```
cargo deb --install
```

### MacOS

- Install alacritty

```
brew install alacritty
```

- Clone the project, then build. Create the config dir and modify the default config file (below) to specify alacritty.

## Build

```sh
cargo build
```

## Configuration

Modify [example config](https://github.com/beeender/glrnvim/blob/master/config.yml) and copy it to your `XDG_CONFIG_HOME` directory.

- For Linux: `$HOME/.config/glrnvim/config.yml`
- For MacOS: `$HOME/Library/Preferences/glrnvim/config.yml`
- For Windows: `{FOLDERID_RoamingAppData}` (`C:\Users\Alice\AppData\Roaming\glrnvim\config.yml`)

## Tips

### Set `glrnvim` as the git editor for commit message

```sh
git config --global core.editor "glrnvim --nofork"
```

### Check if it is running in a glrnvim instance in vim srcipt

```viml
if exists('g:glrnvim_gui')
    "do something
endif
```

## Known Issues:

_Color scheme doesn't work well with urxvt backend._

glrnvim uses `set termguicolors` to achieve an easy and better color scheme support. However, that requires the terminal to support true colors. Urxvt never has an official release to support it. Although the true color patch has been merged many years ago. If you are using Arch, just install [rxvt-unicode-cvs](https://aur.archlinux.org/packages/rxvt-unicode-cvs) from aur.

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Stanislav-Lapata"><img src="https://avatars1.githubusercontent.com/u/12072329?v=4?s=100" width="100px;" alt="Stanislav Lapata"/><br /><sub><b>Stanislav Lapata</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=Stanislav-Lapata" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/yeyan"><img src="https://avatars1.githubusercontent.com/u/5893217?v=4?s=100" width="100px;" alt="Ye Yan"/><br /><sub><b>Ye Yan</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=yeyan" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/LasseWolter"><img src="https://avatars1.githubusercontent.com/u/29123172?v=4?s=100" width="100px;" alt="Lasse Wolter"/><br /><sub><b>Lasse Wolter</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=LasseWolter" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://twitter.com/#!/ddrcode"><img src="https://avatars1.githubusercontent.com/u/700125?v=4?s=100" width="100px;" alt="David de Rosier"/><br /><sub><b>David de Rosier</b></sub></a><br /><a href="#platform-ddrcode" title="Packaging/porting to new platform">📦</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://jandamm.de"><img src="https://avatars.githubusercontent.com/u/5963139?v=4?s=100" width="100px;" alt="Jan Damm"/><br /><sub><b>Jan Damm</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=jandamm" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/p00f"><img src="https://avatars.githubusercontent.com/u/36493671?v=4?s=100" width="100px;" alt="Chinmay Dalal"/><br /><sub><b>Chinmay Dalal</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=p00f" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://crisidev.org/"><img src="https://avatars.githubusercontent.com/u/1781140?v=4?s=100" width="100px;" alt="Matteo Bigoi"/><br /><sub><b>Matteo Bigoi</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=crisidev" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/LollipopFt"><img src="https://avatars.githubusercontent.com/u/62802897?v=4?s=100" width="100px;" alt="LollipopFt"/><br /><sub><b>LollipopFt</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=LollipopFt" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/robacarp"><img src="https://avatars.githubusercontent.com/u/208647?v=4?s=100" width="100px;" alt="robacarp"/><br /><sub><b>robacarp</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=robacarp" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://higuoxing.com"><img src="https://avatars.githubusercontent.com/u/21099318?v=4?s=100" width="100px;" alt="Xing Guo"/><br /><sub><b>Xing Guo</b></sub></a><br /><a href="https://github.com/beeender/glrnvim/commits?author=higuoxing" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!

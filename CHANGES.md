CHANGES
--------

Next Release

-

1.6.0

- Add `omit_term_stderr` to the config to disable unnecessary backend terminal output to the stderr.

1.5.0

- Fix a CWD issue with wezterm backend. (#75)
- Add [foot](https://codeberg.org/dnkl/foot) support. (#77)

1.4.0

- Fix issues with alacritty `0.13.0` by using toml config file. `alacritty < 0.13.0` won't be supported anymore.


1.3.2

- Allow to specify a configuration file path for Kitty, Wezterm and Alacritty inside
  config.yml

1.3.1

- Fix nvim start args for Windows. (#45)
- Show the config path hint in `help`.
- Fix nvim resizing problem with alacritty at the star time. (#52)
- Update dependencies. (#53)

1.3.0

- [Deprecated] The config path is changed from `$HOME/config/glrnvim.yml` to `$HOME/config/glrnvim/config.yml`. The old config path is still supported with a lower priority.
- [Deprecated] Allow to specify nvim path using `nvim_exe_path` config option. Deprecate `exe_path` while letting it still work with a lower priority.

1.2.0

- Add [wezterm](https://wezfurlong.org/wezterm/) support. (#40)

1.1.1

- Add `g:glrnvim_gui=1` to the VIML environment. (#20)
- Fix windows build by using `which` instead of `quale`. (#19)
- Fix `serde_yaml` panic which is caused by rust update. (#26)

1.1.0

- Add `load_term_conf` option for loading terminal's default configurations before applying glrnvim's settings to terminal. (#8)

1.0.0

- **BREAKING** Config file format changed.
- Support more terminals, kitty and urxvt.

0.1.2

- Fix display issues when start from iTerm on MacOS.

0.1.1 20190327

- Add `--nofork`.


0.1.0 20190321

- First release.

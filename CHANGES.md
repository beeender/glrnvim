CHANGES
--------

Next Release

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

use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::{thread, time};
use sysinfo::{Pid, ProcessExt, Signal, System, SystemExt};
use tempfile::NamedTempFile;
extern crate log;
extern crate serde_yaml;

pub const ALACRITTY_NAME: &str = "alacritty";

struct Alacritty {
    exe_path: PathBuf,
    cfg_file: Option<NamedTempFile>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.term_exe_path, ALACRITTY_NAME)?;

    Ok(Box::new(Alacritty {
        exe_path,
        cfg_file: None,
    }))
}

impl Alacritty {
    fn create_base_conf(&mut self, config: &Config) -> serde_yaml::Mapping {
        config.term_config_path.as_ref().map_or_else(
            {
                || match config.load_term_conf {
                    true => Alacritty::load_alacritty_conf(None),
                    _ => serde_yaml::Mapping::new(),
                }
            },
            |p| Alacritty::load_alacritty_conf(Some(p.to_string())),
        )
    }

    fn create_conf_file(
        &mut self,
        base_mapping: &mut serde_yaml::Mapping,
        config: &Config,
    ) {
        let key_font = serde_yaml::to_value("font").unwrap();
        if !base_mapping.contains_key(&key_font) {
            // Try to merge the terminal settings
            let v = serde_yaml::to_value(serde_yaml::Mapping::new()).unwrap();
            base_mapping.insert(key_font.clone(), v);
        }
        // Set the font size
        let font_mapping = base_mapping
            .get_mut(&key_font)
            .unwrap()
            .as_mapping_mut()
            .unwrap();
        if config.font_size > 0 {
            font_mapping.insert(
                serde_yaml::to_value("size").unwrap(),
                serde_yaml::to_value(config.font_size).unwrap(),
            );
        }
        // Set the font
        if !config.fonts.is_empty() {
            let mut normal_mapping = serde_yaml::Mapping::new();
            normal_mapping.insert(
                serde_yaml::to_value("family").unwrap(),
                serde_yaml::to_value(&config.fonts.first()).unwrap(),
            );
            font_mapping.insert(
                serde_yaml::to_value("normal").unwrap(),
                serde_yaml::to_value(normal_mapping).unwrap(),
            );
        }
        // Only overwrite the font setting if it has been set in the glrnvim config
        if font_mapping.is_empty() {
            base_mapping.remove(&key_font);
        }

        // Disable some improper key bindings for nvim
        let binding: serde_yaml::Value =
            serde_yaml::from_str(r#"{key: Z, mods: Control, action: None}"#).unwrap();
        let key_bindings = vec![binding];
        base_mapping.insert(
            serde_yaml::to_value("key_bindings").unwrap(),
            serde_yaml::to_value(key_bindings).unwrap(),
        );

        let yml_str = serde_yaml::to_string(&base_mapping).unwrap();
        // Add single quote to hex color values like:
        // background: 0xf1f1f1 -> background: '0xf1f1f1'
        // Otherwise Alacritty will fail to parse the config since the
        // hex value will be treated as numbers.
        let re = Regex::new(r"(?P<h>.*: *)(?P<x>0x.*)").unwrap();
        let after = re.replace_all(yml_str.as_str(), "$h'$x'");

        // Write to a temp file to be loaded by alacritty
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(&file, after.as_bytes()).unwrap();

        self.cfg_file = Some(file);
    }

    // Load the default alacritty config
    fn load_alacritty_conf(path: Option<String>) -> serde_yaml::Mapping {
        let conf_path = path.or({
            let base_confs: [String; 0] = [];
            let pri_confs: [String; 3] = [
                "$XDG_CONFIG_HOME/alacritty/alacritty.yml".to_string(),
                "$HOME/.config/alacritty/alacritty.yml".to_string(),
                "$XDG_CONFIG_DIRS/alacritty/alacritty.yml".to_string(),
            ];
            let confs = super::find_term_conf_files(&base_confs, &pri_confs);
            Some(confs[0].to_string())
        });
        match conf_path {
            Some(p) => {
                let file = std::fs::File::open(p).unwrap();
                let reader = std::io::BufReader::new(file);
                match serde_yaml::from_reader(reader) {
                    Ok(mapping) => mapping,
                    Err(_) => serde_yaml::Mapping::new(),
                }
            }
            _ => serde_yaml::Mapping::new(),
        }
    }
}

impl Functions for Alacritty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {

        let mut base_conf = self.create_base_conf(config);

        self.create_conf_file(&mut base_conf, config);
        let mut command = std::process::Command::new(&self.exe_path);
        command.arg("--config-file");
        command.arg(self.cfg_file.as_ref().unwrap().path());
        command.arg("--class");
        command.arg("glrnvim");

        if let Ok(current_dir) = std::env::current_dir() {
            command.arg("--working-directory");
            command.arg(current_dir);
        }

        command.arg("-e");
        command.arg(&config.nvim_exe_path);
        command.args(super::COMMON_ARGS);

        command
    }

    // To work around the neovim window size problem while starting.
    // See https://github.com/neovim/neovim/issues/11330
    #[cfg(target_os = "linux")]
    fn post_start(&mut self, config: &Config, term_pid: Pid) {
        let proc_name = match Path::new(&config.nvim_exe_path)
            .file_name()
            .and_then(OsStr::to_str)
        {
            None => {
                log::warn!(
                    "Cannot identify executable name from '{}'",
                    config.nvim_exe_path
                );
                return;
            }
            Some(name) => name,
        };

        let mut count = 0u32;
        let ten_millis = time::Duration::from_millis(10);
        loop {
            count += 1;
            let s = System::new_all();
            for process in s.process_by_name(proc_name) {
                match process.parent() {
                    None => {}
                    Some(ppid) => {
                        if ppid == term_pid {
                            process.kill_with(Signal::Winch);
                            return;
                        }
                    }
                }
            }
            if count == 10 {
                log::warn!("Failed to try resizing the neovim window");
                break;
            }
            thread::sleep(ten_millis);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::fs;
    use std::io::{BufWriter, Write};
    use std::path::PathBuf;

    #[test]
    fn test_create_basic_alacritty_conf() {
        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            term_exe_path: None,
            term_config_path: None,
            exe_path: None,
            nvim_exe_path: "nvim".to_owned(),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            load_term_conf: false,
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut serde_yaml::Mapping::new(), &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"---
font:
  size: 14
  normal:
    family: test_font
key_bindings:
  - key: Z
    mods: Control
    action: None
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_overwrite_alacritty_conf() {
        let mut term_conf = serde_yaml::Mapping::new();
        let mut font_mapping = serde_yaml::Mapping::new();
        font_mapping.insert(
            serde_yaml::to_value("size").unwrap(),
            serde_yaml::to_value(42).unwrap(),
        );
        term_conf.insert(
            serde_yaml::to_value("font").unwrap(),
            serde_yaml::to_value(font_mapping).unwrap(),
        );

        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            term_exe_path: None,
            term_config_path: None,
            exe_path: None,
            nvim_exe_path: "nvim".to_owned(),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            load_term_conf: false,
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut term_conf, &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"---
font:
  size: 14
  normal:
    family: test_font
key_bindings:
  - key: Z
    mods: Control
    action: None
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_not_overwrite_alacritty_conf() {
        let mut term_conf = serde_yaml::Mapping::new();
        let mut font_mapping = serde_yaml::Mapping::new();
        font_mapping.insert(
            serde_yaml::to_value("size").unwrap(),
            serde_yaml::to_value(42).unwrap(),
        );
        term_conf.insert(
            serde_yaml::to_value("font").unwrap(),
            serde_yaml::to_value(font_mapping).unwrap(),
        );

        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            term_exe_path: None,
            term_config_path: None,
            exe_path: None,
            nvim_exe_path: "nvim".to_owned(),
            font_size: 0,
            fonts: vec![],
            load_term_conf: false,
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut term_conf, &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"---
font:
  size: 42
key_bindings:
  - key: Z
    mods: Control
    action: None
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_hex_value_serialize() {
        let mut term_conf = serde_yaml::Mapping::new();
        let mut primary = serde_yaml::Mapping::new();
        let mut colors = serde_yaml::Mapping::new();
        primary.insert(
            serde_yaml::to_value("background").unwrap(),
            serde_yaml::to_value("0x424242").unwrap(),
        );
        colors.insert(
            serde_yaml::to_value("primary").unwrap(),
            serde_yaml::to_value(primary).unwrap(),
        );
        term_conf.insert(
            serde_yaml::to_value("colors").unwrap(),
            serde_yaml::to_value(colors).unwrap(),
        );

        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            term_exe_path: None,
            term_config_path: None,
            exe_path: None,
            nvim_exe_path: "nvim".to_owned(),
            font_size: 0,
            fonts: vec![],
            load_term_conf: false,
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut term_conf, &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"---
colors:
  primary:
    background: "0x424242"
key_bindings:
  - key: Z
    mods: Control
    action: None
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_term_config_path() {
        let term_conf = r#"
env:
  TERM: some
font:
  size: 16
"#;
        let term_conf_file = tempfile::NamedTempFile::new().unwrap();
        let mut w = BufWriter::new(&term_conf_file);
        w.write_all(term_conf.as_bytes()).ok();
        w.flush().expect("");
        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            term_exe_path: None,
            term_config_path: Some(term_conf_file.path().to_str().unwrap().to_string()),
            exe_path: None,
            nvim_exe_path: "nvim".to_owned(),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            load_term_conf: false,
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        let mut base_conf = alacritty.create_base_conf(&conf);
        alacritty.create_conf_file(&mut base_conf, &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"---
env:
  TERM: some
font:
  size: 14
  normal:
    family: test_font
key_bindings:
  - key: Z
    mods: Control
    action: None
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }
}

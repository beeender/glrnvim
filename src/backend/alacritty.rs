use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::{thread, time};
use sysinfo::{Pid, Signal, System};
use tempfile::NamedTempFile;
extern crate log;
use toml_edit::{value, DocumentMut, Item, Table, Value};

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
    fn create_base_conf(&mut self, config: &Config) -> DocumentMut {
        config.term_config_path.as_ref().map_or_else(
            {
                || match config.load_term_conf {
                    true => Alacritty::load_alacritty_conf(None),
                    _ => DocumentMut::new(),
                }
            },
            |p| Alacritty::load_alacritty_conf(Some(p.to_string())),
        )
    }

    fn create_conf_file(&mut self, base_mapping: &mut DocumentMut, config: &Config) {
        let key_font = "font";
        if !base_mapping.contains_key(key_font) {
            // Try to merge the terminal settings
            let v = Table::new();
            base_mapping[key_font] = Item::Table(v);
        }
        // Set the font size
        let font_mapping = base_mapping
            .get_mut(key_font)
            .unwrap()
            .as_table_mut()
            .unwrap();
        if config.font_size > 0 {
            font_mapping.insert("size", value(Into::<i64>::into(config.font_size)));
        }
        // Set the font
        if !config.fonts.is_empty() {
            let mut normal_mapping = Table::new();
            normal_mapping.insert("family", value(config.fonts.first().unwrap().to_string()));
            font_mapping.insert("normal", Item::Table(normal_mapping));
        }
        // Only overwrite the font setting if it has been set in the glrnvim config
        if font_mapping.is_empty() {
            base_mapping.remove(key_font);
        }

        // Disable some improper key bindings for nvim
        let mut binding_z = toml_edit::InlineTable::new();
        binding_z.insert("key", Value::from("Z"));
        binding_z.insert("mods", Value::from("Control"));
        binding_z.insert("action", Value::from("None"));
        let mut bindings = toml_edit::Array::new();
        bindings.push(binding_z);
        let mut keyboard = Table::new();
        keyboard["bindings"] = toml_edit::Item::Value(Value::Array(bindings));
        base_mapping["keyboard"] = Item::Table(keyboard);

        let toml_str = base_mapping.to_string();

        // Write to a temp file to be loaded by alacritty
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(&file, toml_str.as_bytes()).unwrap();

        self.cfg_file = Some(file);
    }

    // Load the default alacritty config
    fn load_alacritty_conf(path: Option<String>) -> DocumentMut {
        let conf_path = path.or({
            let base_confs: [String; 0] = [];
            let pri_confs: [String; 3] = [
                "$XDG_CONFIG_HOME/alacritty/alacritty.toml".to_string(),
                "$HOME/.config/alacritty/alacritty.toml".to_string(),
                "$XDG_CONFIG_DIRS/alacritty/alacritty.toml".to_string(),
            ];
            let confs = super::find_term_conf_files(&base_confs, &pri_confs);
            if confs.is_empty() {
                None
            } else {
                Some(confs[0].to_string())
            }
        });
        match conf_path {
            Some(p) => {
                let content = std::fs::read_to_string(p.clone())
                    .expect(format!("Cannot load term config file: '{}'", p).as_ref());
                match content.parse::<DocumentMut>() {
                    Ok(mapping) => mapping,
                    Err(msg) => {
                        log::warn!("Cannot identify executable name from '{}'", msg);
                        DocumentMut::new()
                    }
                }
            }
            _ => DocumentMut::new(),
        }
    }
}

impl Functions for Alacritty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        let mut doc = self.create_base_conf(config);

        self.create_conf_file(&mut doc, config);
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
        let proc_name = match Path::new(&config.nvim_exe_path).file_name() {
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
            for process in s.processes_by_name(proc_name) {
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
            backend: Some(config::Backend::Alacritty),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            ..Default::default()
        };
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut DocumentMut::new(), &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"[font]
size = 14

[font.normal]
family = "test_font"

[keyboard]
bindings = [{ key = "Z", mods = "Control", action = "None" }]
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_overwrite_alacritty_conf() {
        let mut term_conf = DocumentMut::new();
        let mut font_mapping = Table::new();
        font_mapping.insert("size", value(42));
        term_conf.insert("font", Item::Table(font_mapping));

        let conf = config::Config {
            backend: Some(config::Backend::Alacritty),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            ..Default::default()
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
        let expected = r#"[font]
size = 14

[font.normal]
family = "test_font"

[keyboard]
bindings = [{ key = "Z", mods = "Control", action = "None" }]
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_not_overwrite_alacritty_conf() {
        let mut term_conf = DocumentMut::new();
        let mut font_mapping = Table::new();
        font_mapping.insert("size", value(42));
        term_conf.insert("font", Item::Table(font_mapping));

        let mut conf = config::Config::default();
        conf.backend = Some(config::Backend::Alacritty);
        let mut alacritty = Alacritty {
            exe_path: PathBuf::new(),
            cfg_file: None,
        };
        alacritty.create_conf_file(&mut term_conf, &conf);
        let tmp_conf = alacritty.cfg_file;
        assert!(tmp_conf.is_some());
        let result = fs::read_to_string(tmp_conf.as_ref().unwrap().path());
        assert!(result.is_ok());
        let expected = r#"[font]
size = 42

[keyboard]
bindings = [{ key = "Z", mods = "Control", action = "None" }]
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_hex_value_serialize() {
        let mut term_conf = DocumentMut::new();
        let mut primary = Table::new();
        let mut colors = Table::new();
        primary.insert("background", value("0x424242".to_string()));
        colors.insert("primary", Item::Table(primary));
        term_conf.insert("colors", Item::Table(colors));

        let conf = config::Config {
            backend: Some(config::Backend::Alacritty),
            ..Default::default()
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
        let expected = r#"[colors]

[colors.primary]
background = "0x424242"

[keyboard]
bindings = [{ key = "Z", mods = "Control", action = "None" }]
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }

    #[test]
    fn test_term_config_path() {
        let term_conf = r#"[env]
TERM = "some"

[font]
size = 16
"#;
        let term_conf_file = tempfile::NamedTempFile::new().unwrap();
        let mut w = BufWriter::new(&term_conf_file);
        w.write_all(term_conf.as_bytes()).ok();
        w.flush().expect("");
        let conf = config::Config {
            backend: Some(config::Backend::Alacritty),
            term_config_path: Some(term_conf_file.path().to_str().unwrap().to_string()),
            font_size: 14,
            fonts: vec!["test_font".to_string()],
            ..Default::default()
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
        let expected = r#"[env]
TERM = "some"

[font]
size = 14

[font.normal]
family = "test_font"

[keyboard]
bindings = [{ key = "Z", mods = "Control", action = "None" }]
"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }
}

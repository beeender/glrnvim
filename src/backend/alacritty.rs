use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use crate::NVIM_NAME;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;
extern crate serde_yaml;

pub const ALACRITTY_NAME: &str = "alacritty";

struct Alacritty {
    exe_path: PathBuf,
    cfg_file: Option<NamedTempFile>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.exe_path, ALACRITTY_NAME)?;

    Ok(Box::new(Alacritty {
        exe_path,
        cfg_file: None,
    }))
}

impl Alacritty {
    fn create_conf_file(&mut self, base_mapping: &mut serde_yaml::Mapping, config: &Config) {
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
            for font in &config.fonts {
                normal_mapping.insert(
                    serde_yaml::to_value("family").unwrap(),
                    serde_yaml::to_value(font).unwrap(),
                );
                // TODO: Alacritty doesn't support fallback font well.
                // See https://github.com/jwilm/alacritty/issues/957
                break;
            }
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
    fn load_alacritty_conf() -> serde_yaml::Mapping {
        let base_confs: [String; 0] = [];
        let pri_confs: [String; 3] = [
            "$XDG_CONFIG_HOME/alacritty/alacritty.yml".to_string(),
            "$HOME/.config/alacritty/alacritty.yml".to_string(),
            "$XDG_CONFIG_DIRS/alacritty/alacritty.yml".to_string(),
        ];
        let confs = super::find_term_conf_files(&base_confs, &pri_confs);
        if confs.len() > 0 {
            let path = confs[0].to_owned();
            let file = std::fs::File::open(path).unwrap();
            let reader = std::io::BufReader::new(file);
            match serde_yaml::from_reader(reader) {
                Ok(mapping) => mapping,
                Err(_) => serde_yaml::Mapping::new(),
            }
        } else {
            serde_yaml::Mapping::new()
        }
    }
}

impl Functions for Alacritty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        let mut base_conf = match config.load_term_conf {
            true => Alacritty::load_alacritty_conf(),
            _ => serde_yaml::Mapping::new(),
        };

        self.create_conf_file(&mut base_conf, config);
        let mut command = std::process::Command::new(self.exe_path.to_owned());
        command.arg("--config-file");
        command.arg(self.cfg_file.as_ref().unwrap().path());
        command.arg("--class");
        command.arg("glrnvim");

        if let Ok(current_dir) = std::env::current_dir() {
            command.arg("--working-directory");
            command.arg(current_dir);
        }

        command.arg("-e");
        command.arg(NVIM_NAME);

        // Enable 24-bits colors
        command.arg("+set termguicolors");
        // Set title string
        command.arg("+set title");
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_create_basic_alacritty_conf() {
        let conf = config::Config {
            fork: false,
            backend: Some(config::Backend::Alacritty),
            exe_path: None,
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
    action: None"#;
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
            exe_path: None,
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
    action: None"#;
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
            exe_path: None,
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
    action: None"#;
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
            exe_path: None,
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
    background: '0x424242'
key_bindings:
  - key: Z
    mods: Control
    action: None"#;
        assert_eq!(result.unwrap_or_default(), expected)
    }
}

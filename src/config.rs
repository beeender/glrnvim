extern crate serde;
extern crate serde_yaml;

use serde::Deserialize;
use std::path::PathBuf;

const NVIM_NAME: &str = "nvim";

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Alacritty,
    Urxvt,
    Kitty,
    Wezterm,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub fork: bool,
    pub backend: Option<Backend>,
    // TODO: this config option is deprecated, will be removed in the future
    pub exe_path: Option<String>,
    pub term_exe_path: Option<String>,
    pub term_config_path: Option<String>,
    #[serde(default)]
    pub nvim_exe_path: String,
    #[serde(default)]
    pub load_term_conf: bool,
    #[serde(default)]
    pub fonts: Vec<String>,
    #[serde(default)]
    pub font_size: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fork: false,
            backend: None,
            nvim_exe_path: NVIM_NAME.to_owned(),
            exe_path: None,
            term_exe_path: None,
            term_config_path: None,
            fonts: Vec::new(),
            font_size: 0,
            load_term_conf: false,
        }
    }
}

pub fn parse(path: PathBuf) -> Config {
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut config = match serde_yaml::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            // Work around the empty yaml file issue.
            // See https://github.com/dtolnay/serde-yaml/issues/86
            if e.to_string() != "EOF while parsing a value" {
                panic!("{}", e.to_string())
            }
            Config::default()
        }
    };

    if config.backend.is_none() && config.term_exe_path.is_some() {
        panic!("term_exe_path requires a backend key")
    }

    if config.nvim_exe_path.is_empty() {
        config.nvim_exe_path = NVIM_NAME.to_owned()
    }

    // TODO: this config option is deprecated, will be removed in the future
    if config.exe_path.is_some() {
        config.term_exe_path = config.exe_path.clone()
    }

    config.fonts = config
        .fonts
        .into_iter()
        .filter(|s| !s.is_empty() && s != "~")
        .collect::<Vec<_>>();

    config
}

pub fn complete(mut config: Config, fork: bool) -> Config {
    config.fork = fork;
    config
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    struct TempConfFile {
        _dir: TempDir,
        path: PathBuf,
    }

    fn make_cfg_file(content: &str) -> TempConfFile {
        // Create a directory inside of `std::env::temp_dir()`.
        let dir = tempdir().unwrap();

        let file_path = dir.path().join("glrnvim.yaml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        drop(file);
        TempConfFile {
            _dir: dir,
            path: file_path,
        }
    }

    #[test]
    fn test_parse_fonts() {
        let config = parse(
            make_cfg_file(
                r#"
fonts:
  - MonoAbc ff
  -
  - ~
  - ac
"#,
            )
            .path,
        );
        assert_eq!(config.fonts.len(), 2);
        assert_eq!(config.fonts, vec!["MonoAbc ff", "ac"]);
    }

    #[test]
    fn test_parse_font_size() {
        let config = parse(make_cfg_file("font_size: 15").path);
        assert_eq!(config.font_size, 15);
        assert!(config.fonts.is_empty());
    }

    #[test]
    fn test_parse_empty_config() {
        let config = parse(make_cfg_file("").path);
        assert_eq!(config, Config::default());
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_parse_non_exist_config() {
        parse(PathBuf::from("/non/exists"));
    }

    #[test]
    fn test_parse_backend_and_term_exe_path() {
        let config =
            parse(make_cfg_file("backend: alacritty\nterm_exe_path: /path/to/alacritty").path);
        assert_eq!(config.backend, Some(Backend::Alacritty));
        assert_eq!(config.term_exe_path, Some("/path/to/alacritty".to_string()));
    }

    #[test]
    fn test_parse_backend_and_deprecated_exe_path() {
        let config = parse(make_cfg_file("backend: alacritty\nexe_path: /path/to/alacritty").path);
        assert_eq!(config.backend, Some(Backend::Alacritty));
        assert_eq!(config.term_exe_path, Some("/path/to/alacritty".to_string()));
    }

    #[test]
    #[should_panic(expected = "exe_path requires a backend key")]
    fn test_parse_exe_path_without_backend() {
        parse(make_cfg_file("term_exe_path: /path/to/kitty").path);
    }

    #[test]
    #[should_panic(
        expected = "font_size: invalid type: string \"sadfa\", expected u8 at line 1 column 12"
    )]
    fn test_parse_invalid_font_size() {
        parse(make_cfg_file("font_size: sadfa").path);
    }
}

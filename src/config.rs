extern crate serde;
extern crate serde_yaml;

use serde::Deserialize;

const DEFAULT_FONT_SIZE: u8 = 12;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub fork: bool,
    pub backend: Option<String>,
    #[serde(default)]
    pub fonts: Vec<String>,
    #[serde(default = "default_font_size")]
    pub font_size: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fork: false,
            backend: None,
            fonts: Vec::new(),
            font_size: DEFAULT_FONT_SIZE,
        }
    }
}

#[allow(dead_code)]
fn default_font_size() -> u8 {
    DEFAULT_FONT_SIZE
}

pub fn parse(path: &str, config: &mut Config) {
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    let conf: Config = match serde_yaml::from_reader(reader) {
        Ok(c) => c,
        Err(e) => {
            // Work around the empty yaml file issue.
            // See https://github.com/dtolnay/serde-yaml/issues/86
            if e.to_string() == "EOF while parsing a value" {
                Config::default()
            } else {
                panic!(e.to_string())
            }
        }
    };

    config.backend = conf.backend;
    // Filter out empty strings
    config.fonts = conf
        .fonts
        .into_iter()
        .filter(|s| !s.is_empty() && s != "~")
        .collect::<Vec<_>>();
    config.font_size = conf.font_size;
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
        path: String,
    }

    fn make_cfg_file(content: &str) -> TempConfFile {
        // Create a directory inside of `std::env::temp_dir()`.
        let dir = tempdir().unwrap();

        let file_path = dir.path().join("glrnvim.config");
        let mut file = File::create(file_path.to_owned()).unwrap();
        file.write(content.as_bytes()).unwrap();
        file.flush().unwrap();
        drop(file);
        TempConfFile {
            _dir: dir,
            path: file_path.into_os_string().into_string().unwrap(),
        }
    }

    #[test]
    fn test_parse() {
        let mut config = Config {
            fork: false,
            backend: None,
            fonts: vec![],
            font_size: 0,
        };

        parse(
            &make_cfg_file(
                r#"
fonts:
  - MonoAbc ff
  -
  - ac
"#,
            )
            .path,
            &mut config,
        );
        assert!(config.fonts[0] == "MonoAbc ff");
        assert!(config.fonts[1] == "ac");
        assert!(config.font_size == 12);
        assert!(config.backend.is_none());

        parse(&make_cfg_file("font_size: 15").path, &mut config);
        assert!(config.font_size == 15);
        assert!(config.fonts.is_empty());

        parse(&make_cfg_file("backend: kitty").path, &mut config);
        assert!(config.backend == Some("kitty".to_string()));

        // Empty config
        parse(&make_cfg_file("").path, &mut config);
        assert!(config.backend.is_none());
        assert!(config.fonts.is_empty());
        assert!(config.font_size == 12);

        let result = std::panic::catch_unwind(|| {
            let mut conf = std::panic::AssertUnwindSafe(config);
            parse(&make_cfg_file("font_size: sadfa").path, &mut conf)
        });
        assert!(result.is_err());
    }
}

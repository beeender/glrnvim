use super::Functions;
use crate::config::Config;
use crate::NVIM_NAME;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub const ALACRITTY_NAME: &str = "alacritty";

struct Alacritty {
    exe_path: PathBuf,
    temp_file: Option<NamedTempFile>,
}

pub fn init() -> Result<Box<dyn Functions>, String> {
    match super::find_executable(ALACRITTY_NAME) {
        Ok(p) => {
            return Ok(Box::new(Alacritty {
                exe_path: p,
                temp_file: None,
            }));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

impl Alacritty {
    fn create_conf_file(&mut self, config: &Config) {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "font:").unwrap();
        writeln!(file, "  size: {}", config.font_size).unwrap();

        if !config.fonts.is_empty() {
            writeln!(file, "  normal:").unwrap();
        }
        for font in &config.fonts {
            writeln!(file, "    family: \"{}\"", font).unwrap();
            // TODO: Alacritty doesn't support fallback font well.
            // See https://github.com/jwilm/alacritty/issues/957
            break;
        }

        writeln!(file, "key_bindings:").unwrap();
        writeln!(file, "  - {{key: Z, mods: Control, action: None}} ").unwrap();
        file.flush().unwrap();

        file.path();
        self.temp_file = Some(file);
    }
}

impl Functions for Alacritty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.create_conf_file(config);
        let mut command = std::process::Command::new(self.exe_path.to_owned());
        command.arg("--config-file");
        command.arg(self.temp_file.as_ref().unwrap().path());
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
        return command;
    }
}

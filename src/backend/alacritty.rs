use std::path::{PathBuf};
use super::Functions;
use crate::NVIM_NAME;
use crate::config::Config;
use tempfile::NamedTempFile;
use std::io::Write;

#[cfg(target_os = "macos")]
use std::path::Path;

pub const ALACRITTY_NAME: &str = "alacritty";

struct Alacritty {
    exe_path: PathBuf,
    temp_file: Option<NamedTempFile>
}

#[cfg(not(target_os = "macos"))]
pub fn init() -> Result<impl Functions, String> {
    match quale::which(ALACRITTY_NAME) {
        Some(p) => {
            return Ok(Alacritty {exe_path: p, temp_file: None});
        }
        _ => {
            return Err("'alacritty' executable cannot be found.".to_owned());
        }
    }
}

#[cfg(target_os = "macos")]
pub fn init() -> Result<impl Functions, String> {
    match quale::which(ALACRITTY_NAME) {
        Some(p) => {
            return Ok(Alacritty {exe_path: p});
        }
        _ => {}
    }

    let app_path = Path::new("/Applications/Alacritty.app/Contents/MacOS/");
    let exe_path = app_path.join(ALACRITTY_NAME);
    if exe_path.exists() && exe_path.is_file() {
        return Ok(Alacritty {exe_path});
    }

    match dirs::home_dir() {
        Some(home) => {
            let exe_path = home.join("Alacritty.app/Contents/MacOS/").join(ALACRITTY_NAME);
            if exe_path.exists() && exe_path.is_file() {
                return Ok(Alacritty {exe_path});
            }
        }
        _ => {}
    }

    return Err("'alacritty' executable cannot be found.".to_owned());
}

impl Alacritty { fn create_conf_file(&mut self, config: &Config)  {
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
        command.arg("-e");
        command.arg(NVIM_NAME);

        // Enable 24-bits colors
        command.arg("+set termguicolors");
        // Set title string
        command.arg("+set title");
        return command;
    }
}


use std::path::{PathBuf};
use super::Functions;
use crate::NVIM_NAME;

#[cfg(target_os = "macos")]
use std::path::Path;

pub const ALACRITTY_NAME: &str = "alacritty";

struct Alacritty {
    exe_path: PathBuf
}

#[cfg(not(target_os = "macos"))]
pub fn init() -> Result<impl Functions, String> {
    match quale::which(ALACRITTY_NAME) {
        Some(p) => {
            return Ok(Alacritty {exe_path: p});
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

impl Functions for Alacritty {
    fn create_command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(self.exe_path.to_owned());
        let config_dir = dirs::config_dir();
        match config_dir {
            Some(mut dir) => {
                dir.push("glrnvim.yml");
                if dir.as_path().exists() {
                    command.arg("--config-file");
                    command.arg(dir.as_path().as_os_str());
                }
            }
            _ => {}
        };
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


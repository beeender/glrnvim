mod alacritty;
use super::config::Config;
use std::path::PathBuf;

pub trait Functions {
    fn create_command(&mut self, config: &Config) -> std::process::Command;
}

pub fn init(backend_name: &str) -> Result<impl Functions, String> {
    if backend_name.to_lowercase() == alacritty::ALACRITTY_NAME {
        return alacritty::init();
    }

    return Err(format!("Backend terminal '{}' is not supported.",
            backend_name));
}

#[cfg(not(target_os = "macos"))]
fn find_executable(exe_name: &str) -> Result<PathBuf, String> {
    match quale::which(exe_name) {
        Some(p) => {
            return Ok(p);
        }
        _ => {
            return Err(format!("'{}' executable cannot be found.", exe_name));
        }
    }
}

#[cfg(target_os = "macos")]
fn find_executable(exe_name: &str) -> Result<PathBuf, String> {
    match quale::which(exe_name) {
        Some(p) => {
            return Ok(p);
        }
        _ => {}
    }

    let mut app_name = exe_name.to_owned();
    if let Some(s) = app_name.get_mut(0..1) {
         s.make_ascii_uppercase();
    }

    let dir = format!("/Applications/{}.app/Contents/MacOS/", app_name);
    let app_path = std::path::Path::new(dir.as_str());
    let exe_path = app_path.join(exe_name);
    if exe_path.exists() && exe_path.is_file() {
        return Ok(exe_path);
    }

    match dirs::home_dir() {
        Some(home) => {
            let exe_path = home.join(format!("{}.app/Contents/MacOS/", app_name)).join(exe_name);
            if exe_path.exists() && exe_path.is_file() {
                return Ok(exe_path);
            }
        }
        _ => {}
    }

    return Err("'alacritty' executable cannot be found.".to_owned());
}

mod alacritty;
mod kitty;
mod urxvt;
use super::config::Config;
use crate::error::GlrnvimError;
use std::path::PathBuf;

pub const BACKEND_LIST: &'static [&'static str] = &[
    alacritty::ALACRITTY_NAME,
    urxvt::URXVT_NAME,
    kitty::KITTY_NAME,
];

pub trait Functions {
    fn create_command(&mut self, config: &Config) -> std::process::Command;
}

pub fn init(backend_name: &str) -> Result<Box<dyn Functions>, GlrnvimError> {
    let name = backend_name.to_lowercase();

    return match name.as_str() {
        alacritty::ALACRITTY_NAME => alacritty::init(),
        urxvt::URXVT_NAME => urxvt::init(),
        kitty::KITTY_NAME => kitty::init(),
        _ => Err(GlrnvimError::new(format!(
            "Backend terminal '{}' is not supported.",
            backend_name
        ))),
    };
}

#[cfg(not(target_os = "macos"))]
fn find_executable(exe_name: &str) -> Result<PathBuf, GlrnvimError> {
    match quale::which(exe_name) {
        Some(p) => Ok(p),
        None => Err(GlrnvimError::new(format!(
            "'{}' executable cannot be found.",
            exe_name
        ))),
    }
}

#[cfg(target_os = "macos")]
fn find_executable(exe_name: &str) -> Result<PathBuf, GlrnvimError> {
    if let Some(p) = quale::which(exe_name) {
        return Ok(p);
    }

    let mut app_name = exe_name.to_owned();
    if let Some(s) = app_name.get_mut(0..1) {
        s.make_ascii_uppercase();
    }

    let exe_path = format!("/Applications/{}.app/Contents/MacOS/{}", app_name, exe_name);
    let exe_path = std::path::Path::new(exe_path.as_str());
    if exe_path.exists() && exe_path.is_file() {
        return Ok(exe_path.to_path_buf());
    }

    if let Some(home) = dirs::home_dir() {
        let exe_path = home.join(format!("{}.app/Contents/MacOS/{}", app_name, app_name));
        if exe_path.exists() && exe_path.is_file() {
            return Ok(exe_path);
        }
    }

    Err(GlrnvimError::new(format!(
        "'{}' executable cannot be found.",
        exe_name
    )))
}

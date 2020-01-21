mod alacritty;
mod kitty;
mod urxvt;
use super::config::Config;
use crate::config::Backend;
use crate::error::GlrnvimError;
use std::path::PathBuf;
extern crate shellexpand;

pub trait Functions {
    fn create_command(&mut self, config: &Config) -> std::process::Command;
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    match &config.backend {
        Some(backend) => match backend {
            Backend::Alacritty => alacritty::init(config),
            Backend::Urxvt => urxvt::init(config),
            Backend::Kitty => kitty::init(config),
        },
        None => {
            for init_func in &[alacritty::init, urxvt::init, kitty::init] {
                match init_func(config) {
                    Ok(functions) => return Ok(functions),
                    Err(_) => {}
                }
            }

            Err(GlrnvimError::new(format!(
                "None of the suppported terminals can be found.",
            )))
        }
    }
}

fn exe_path(exe_path: &Option<String>, exe_name: &str) -> Result<PathBuf, GlrnvimError> {
    let exe_name = match exe_path {
        Some(exe_path) => PathBuf::from(exe_path),
        None => find_executable(exe_name)?,
    };

    Ok(exe_name)
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

fn find_term_conf_files(base_confs: &[String], priority_confs: &[String]) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();

    for path_str in base_confs {
        let expanded_str = &String::from(shellexpand::full(path_str).unwrap_or_default());
        let conf_path = std::path::Path::new(expanded_str);
        if conf_path.exists() && conf_path.is_file() {
            ret.push(expanded_str.to_owned());
        }
    }

    for path_str in priority_confs {
        let expanded_str = &String::from(shellexpand::full(path_str).unwrap_or_default());
        let conf_path = std::path::Path::new(expanded_str);
        if conf_path.exists() && conf_path.is_file() {
            ret.push(expanded_str.to_owned());
            break;
        }
    }

    return ret;
}

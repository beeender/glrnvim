use super::Functions;
use std::path::{PathBuf};
use crate::config::Config;
use crate::NVIM_NAME;

pub const URXVT_NAME: &str = "urxvt";

struct Urxvt {
    exe_path: PathBuf,
    pub args: Vec<String>
}

pub fn init() -> Result<Box<dyn Functions>, String> {
    match super::find_executable(URXVT_NAME) {
        Ok(p) => {
            return Ok(Box::new(Urxvt {exe_path: p, args: vec![]}));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

impl Urxvt {
    fn init_args(&mut self, config: &Config) {
        let mut fn_arg = String::from("");
        for font in &config.fonts {
            if fn_arg.is_empty() {
                self.args.push(String::from("-fn"));
                fn_arg.push_str(
                    format!("xft:{}:size={}:antialias=true", font, config.font_size).as_str());
            } else {
                fn_arg.push_str(
                    format!(",xft:{}:antialias=true", font).as_str());
            }
        }
        if !fn_arg.is_empty() {
            self.args.push(fn_arg);
        }
    }
}

impl Functions for Urxvt {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.init_args(config);
        let mut command = std::process::Command::new(self.exe_path.to_owned());

        command.arg("-name");
        command.arg("glrnvim");
        // Disable Ctrl-Z. Shouldn't this pass ^Z to nvim??
        command.arg("-keysym.C-z:");
        command.arg("builtin-string:");

        for arg in &self.args {
            command.arg(arg);
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

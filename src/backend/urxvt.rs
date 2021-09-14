use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use std::path::PathBuf;

pub const URXVT_NAME: &str = "urxvt";

struct Urxvt {
    exe_path: PathBuf,
    pub args: Vec<String>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.term_exe_path, URXVT_NAME)?;

    Ok(Box::new(Urxvt {
        exe_path,
        args: vec![],
    }))
}

impl Urxvt {
    fn init_args(&mut self, config: &Config) {
        let mut fn_arg = String::from("");
        for font in &config.fonts {
            if fn_arg.is_empty() {
                self.args.push(String::from("-fn"));
                fn_arg.push_str(
                    format!("xft:{}:size={}:antialias=true", font, config.font_size).as_str(),
                );
            } else {
                fn_arg.push_str(format!(",xft:{}:antialias=true", font).as_str());
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
        command.args(&self.args);
        command.arg("-e");
        command.arg(&config.nvim_exe_path);
        command.args(super::COMMON_ARGS);
        command
    }
}

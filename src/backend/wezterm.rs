use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use crate::NVIM_NAME;
use std::io::Write;
use std::path::PathBuf;

pub const WEZTERM_NAME: &str = "wezterm";

struct Wezterm {
    exe_path: PathBuf,
    pub args: Vec<String>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.exe_path, WEZTERM_NAME)?;

    Ok(Box::new(Wezterm {
        exe_path,
        args: vec![],
    }))
}

impl Wezterm {
    fn init_args(&mut self, config: &Config) {
        let mut fn_arg = String::from("");
        if !&config.fonts.is_empty() {
            self.args.push("--config".to_string());
            fn_arg = String::from("font=require('wezterm').font_with_fallback({");
            for font in &config.fonts {
                fn_arg.push('\"');
                fn_arg.push_str(font);
                fn_arg.push('\"');
                fn_arg.push(',');
            }
            fn_arg.push_str("})");
        }
        if !fn_arg.is_empty() {
            self.args.push(fn_arg);
        }
        if config.font_size != 0 {
            self.args.push("--config".to_string());
            self.args
                .push(format!("font_size={}", config.font_size));
        }
    }
}

impl Functions for Wezterm {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.init_args(config);
        let mut command = std::process::Command::new(self.exe_path.to_owned());
        command.args(&self.args);
        if !config.load_term_conf {
            command.arg("--config-file");
            let mut file = tempfile::NamedTempFile::new().unwrap();
            writeln!(file, "return {{}}").unwrap();
            file.flush().unwrap();
            file.path();
            command.arg(Some(file).as_ref().unwrap().path());
        }
        command.arg("start");
        command.arg("--class");
        command.arg("glrnvim");
        command.arg("--");
        command.arg(NVIM_NAME);
        command.args(super::COMMON_ARGS);
        command
    }
}

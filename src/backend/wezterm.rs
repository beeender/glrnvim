use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub const WEZTERM_NAME: &str = "wezterm";

struct Wezterm {
    exe_path: PathBuf,
    pub args: Vec<String>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.term_exe_path, WEZTERM_NAME)?;

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
            fn_arg = String::from("font = require('wezterm').font_with_fallback({");
            for font in &config.fonts {
                fn_arg = format!("{} \"{}\",", fn_arg, font);
            }
            fn_arg.push_str("})");
        }
        if !fn_arg.is_empty() {
            self.args.push(fn_arg);
        }
        if config.font_size != 0 {
            self.args.push("--config".to_string());
            self.args.push(format!("font_size={}", config.font_size));
        }
        self.args.push("--config".to_string());
        self.args.push("enable_tab_bar = false".to_string());
    }

    fn create_conf_file(&mut self) -> NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "return {{}}").unwrap();
        file.flush().unwrap();
        file
    }
}

impl Functions for Wezterm {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.init_args(config);
        let mut command = std::process::Command::new(&self.exe_path);
        if let Some(config_path) = config.term_config_path.as_ref() {
            command.arg("--config-file");
            command.arg(config_path);
        } else if !config.load_term_conf {
            let temp_file = self.create_conf_file();
            command.arg("--config-file");
            command.arg(temp_file.path());
        }
        command.args(&self.args);
        command.arg("start");
        command.arg("--class");
        command.arg("glrnvim");
        command.arg("--cwd");
        command.arg(".");
        command.arg("--");
        command.arg(&config.nvim_exe_path);
        command.args(super::COMMON_ARGS);
        command
    }
}

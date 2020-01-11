use super::Functions;
use std::path::{PathBuf};
use crate::config::Config;
use crate::NVIM_NAME;
use tempfile::NamedTempFile;
use std::io::Write;

pub const KITTY_NAME: &str = "kitty";

struct Kitty {
    exe_path: PathBuf,
    temp_file: Option<NamedTempFile>
}

pub fn init() -> Result<Box<dyn Functions>, String> {
    match super::find_executable(KITTY_NAME) {
        Ok(p) => {
            return Ok(Box::new(Kitty {exe_path: p, temp_file: None}));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

impl Kitty {
    fn create_conf_file(&mut self, config: &Config)  {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        if !config.fonts.is_empty() {
            // Kitty's font fallback system is based on unicode range which is too
            // difficult to support. Just use the first chosen font.
            writeln!(file, "font_family {}", config.fonts.first().unwrap()).unwrap();
        }
        writeln!(file, "font_size {}", config.font_size).unwrap();

        writeln!(file, "clear_all_shortcuts yes").unwrap();
        // Using no_op to bypass ctrl-z seems not working.
        // So choose a harmless action to bypass ctrl-z.
        writeln!(file, "map ctrl+z change_font_size all 0").unwrap();
        file.flush().unwrap();

        file.path();
        self.temp_file = Some(file);
    }
}

impl Functions for Kitty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.create_conf_file(config);
        let mut command = std::process::Command::new(self.exe_path.to_owned());
        command.arg("--config");
        command.arg(self.temp_file.as_ref().unwrap().path());

        if cfg!(target_os = "linux") {
            command.arg("--class");
            command.arg("glrnvim");
        }

        command.arg(NVIM_NAME);

        // Enable 24-bits colors
        command.arg("+set termguicolors");
        // Set title string
        command.arg("+set title");
        return command;
    }
}

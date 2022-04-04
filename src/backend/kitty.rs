use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub const KITTY_NAME: &str = "kitty";

struct Kitty {
    exe_path: PathBuf,
    temp_file: Option<NamedTempFile>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.term_exe_path, KITTY_NAME)?;

    Ok(Box::new(Kitty {
        exe_path,
        temp_file: None,
    }))
}

impl Kitty {
    fn create_conf_file(&mut self, config: &Config) {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        if !config.fonts.is_empty() {
            // Kitty's font fallback system is based on unicode range which is too
            // difficult to support. Just use the first chosen font.
            writeln!(file, "font_family {}", config.fonts.first().unwrap()).unwrap();
        }
        if config.font_size != 0 {
            writeln!(file, "font_size {}", config.font_size).unwrap();
        }

        if !config.load_term_conf {
            writeln!(file, "clear_all_shortcuts yes").unwrap();
        }
        // Using no_op to bypass ctrl-z seems not working.
        // So choose a harmless action to bypass ctrl-z.
        writeln!(file, "map ctrl+z change_font_size all 0").unwrap();
        file.flush().unwrap();

        file.path();
        self.temp_file = Some(file);
    }

    // See the config merge order in the kitty's man page.
    // /etc/xdg/kitty/kitty.conf is always used with the lowest priority.
    // By the listed orders, one of the following four will be picked if it exists.
    // $KITTY_CONFIG_DIRECTORY
    // $XDG_CONFIG_HOME/kitty/kitty.conf
    // ~/.config/kitty/kitty.conf,
    // $XDG_CONFIG_DIRS/kitty/kitty.conf
    fn find_default_confs() -> Vec<String> {
        let base_confs: [String; 1] = ["/etc/xdg/kitty/kitty.conf".to_string()];
        let pri_confs: [String; 4] = [
            "$KITTY_CONFIG_DIRECTORY/kitty.conf".to_string(),
            "$XDG_CONFIG_HOME/kitty.conf".to_string(),
            "$HOME/.config/kitty/kitty.conf".to_string(),
            "$XDG_CONFIG_DIRS/kitty/kitty.conf".to_string(),
        ];
        super::find_term_conf_files(&base_confs, &pri_confs)
    }
}

impl Functions for Kitty {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        self.create_conf_file(config);

        let mut command = std::process::Command::new(&self.exe_path);

        if let Some(config_path) = config.term_config_path.as_ref() {
            command.arg("--config");
            command.arg(config_path.as_str());
        } else if config.load_term_conf {
            let confs = Kitty::find_default_confs();
            for conf in confs {
                command.arg("--config");
                command.arg(conf);
            }
        }

        // Overwrite the config with the generated settings from glrnvim.yml
        command.arg("--config");
        command.arg(self.temp_file.as_ref().unwrap().path());

        if cfg!(target_os = "linux") {
            command.arg("--class");
            command.arg("glrnvim");
        }

        command.arg(&config.nvim_exe_path);
        command.args(super::COMMON_ARGS);

        command
    }
}

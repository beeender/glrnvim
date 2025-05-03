use super::Functions;
use crate::config::Config;
use crate::error::GlrnvimError;
use ini::Ini;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub const FOOT_NAME: &str = "foot";

struct Foot {
    exe_path: PathBuf,
    temp_file: Option<NamedTempFile>,
}

pub fn init(config: &Config) -> Result<Box<dyn Functions>, GlrnvimError> {
    let exe_path = super::exe_path(&config.term_exe_path, FOOT_NAME)?;

    Ok(Box::new(Foot {
        exe_path,
        temp_file: None,
    }))
}

impl Foot {
    fn create_conf_file(&mut self, config: &Config) {
        let mut foot_conf = if config.load_term_conf {
            let conf_files = Foot::find_default_confs();
            if conf_files.is_empty() {
                Ini::new()
            } else {
                let path = conf_files.first().unwrap();
                Ini::load_from_file(path).expect("Failed to load default config file")
            }
        } else {
            Ini::new()
        };

        let mut font_str = String::new();
        for f in &config.fonts {
            if !font_str.is_empty() {
                font_str += ",";
            }
            font_str += f;
            if config.font_size != 0 {
                font_str += &format!(":size={}", config.font_size).to_string();
            }
        }
        if !font_str.is_empty() {
            foot_conf.with_section(Some("main")).set("font", font_str);
        }
        // Note: The ctrl-z seems to be no-op so we don't have to disable it. Other default
        // key bindings are quite harmless for now. If needed, just disable them in the config file
        // here.

        let mut file = tempfile::NamedTempFile::new().expect("Failed to create temporary file");
        foot_conf
            .write_to_file(&file)
            .expect("Failed to write to temporary file");
        file.flush().unwrap();

        file.path();
        self.temp_file = Some(file);
    }

    fn find_default_confs() -> Vec<String> {
        let base_confs: [String; 0] = [];
        let pri_confs: [String; 3] = [
            "$XDG_CONFIG_HOME/foot/foot.conf".to_string(),
            "$HOME/.config/foot/foot.conf".to_string(),
            "$XDG_CONFIG_DIRS/foot/foot.conf".to_string(),
        ];
        super::find_term_conf_files(&base_confs, &pri_confs)
    }
}

impl Functions for Foot {
    fn create_command(&mut self, config: &Config) -> std::process::Command {
        let mut command = std::process::Command::new(&self.exe_path);

        command.arg("--config");
        if let Some(config_path) = config.term_config_path.as_ref() {
            command.arg(config_path.as_str());
        } else {
            self.create_conf_file(config);
            // Overwrite the config with the generated settings from glrnvim.yml
            command.arg(self.temp_file.as_ref().unwrap().path());
        }

        command.arg("--app-id");
        command.arg("glrnvim");

        command.arg(&config.nvim_exe_path);
        command.args(super::COMMON_ARGS);

        command
    }
}

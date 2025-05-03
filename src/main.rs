extern crate dirs;
extern crate env_logger;
extern crate log;

mod backend;
mod config;
mod error;

use config::*;
use std::env;
use std::process::Command;
use sysinfo::Pid;

const DEFAULT_FONT_SIZE: u8 = 12;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_os = "macos"))]
fn prepare_env() {}

#[cfg(target_os = "macos")]
fn prepare_env() {
    // When starting from iTerm, these env vars could cause some display issues.
    log::debug!("unset $TERM_PROGRAM");
    env::remove_var("TERM_PROGRAM");
    log::debug!("unset $TERM_PROGRAM_VERSION");
    env::remove_var("TERM_PROGRAM_VERSION");
}

fn check_nvim(vim_exe_path: &str) {
    if which::which(vim_exe_path).is_err() {
        eprintln!("'{}' executable cannot be found.", vim_exe_path);
        std::process::exit(-1);
    }
}

fn parse_args() -> (Config, Vec<String>) {
    let args: Vec<String> = env::args().collect();
    let mut n_args: Vec<String> = Vec::new();
    let mut fork: bool = true;

    for arg in args.iter().skip(1) {
        if arg.starts_with("-h") || arg.starts_with("--help") {
            show_help();
            std::process::exit(0);
        } else if arg.starts_with("-v") || arg.starts_with("--version") {
            show_version();
            std::process::exit(0);
        } else if arg == "--nofork" {
            fork = false;
        } else {
            n_args.push(arg.clone());
        }
    }

    let mut config: Config = match dirs::config_dir() {
        Some(conf_dir) => {
            let mut new_conf_path = conf_dir.clone();
            new_conf_path.push("glrnvim");
            new_conf_path.push("config.yml");

            let mut old_conf_path = conf_dir;
            old_conf_path.push("glrnvim.yml");

            if new_conf_path.exists() {
                log::debug!("Use config file: '{:?}'.", new_conf_path);
                config::parse(new_conf_path)
            } else if old_conf_path.exists() {
                log::debug!("Use config file: '{:?}'.", old_conf_path);
                config::parse(old_conf_path)
            } else {
                log::debug!("No config file found. Use default config.");
                Config::default()
            }
        }
        None => Config::default(),
    };
    config = config::complete(config, fork);
    if !config.load_term_conf {
        // Set our default configs if user doesn't use the terminal's conf.
        if config.font_size == 0 {
            config.font_size = DEFAULT_FONT_SIZE;
        }
    }

    (config, n_args)
}

fn show_version() {
    let n_ver_out = Command::new("nvim")
        .arg("-v")
        .output()
        .expect("You have to install a proper nvim.");
    if !n_ver_out.status.success() {
        std::process::exit(-1);
    }
    println!("glrnvim {}\n", VERSION);
    println!("{}", String::from_utf8_lossy(&n_ver_out.stdout));
}

fn show_help() {
    let n_help_out = Command::new("nvim")
        .arg("-h")
        .output()
        .expect("You have to install a proper nvim.");
    if !n_help_out.status.success() {
        std::process::exit(-1);
    }

    let n_help = String::from_utf8_lossy(&n_help_out.stdout);
    let lines = n_help.lines();

    let mut help: Vec<String> = Vec::new();
    let mut option_passed: bool = false;

    help.push("Usage:".to_string());
    for line in lines {
        if line.starts_with("Usage:") {
        } else if line.starts_with("Options:") {
            option_passed = true;
            help.push(line.to_string());
            help.push("  --nofork              Do not fork when starting GUI".to_string());
        } else if !option_passed {
            help.push(line.replace("nvim", "glrnvim"));
        } else {
            help.push(line.to_string());
        }
    }

    for line in help {
        println!("{}", line);
    }

    match dirs::config_dir() {
        Some(mut conf_dir) => {
            conf_dir.push("glrnvim");
            conf_dir.push("config.yml");
            println!("\nConfig file: {}", conf_dir.display());
            println!(
                "See https://github.com/beeender/glrnvim/blob/master/glrnvim.yml for example."
            );
        }
        None => {
            println!("\nConfig file: Cannot identify the current config directory. No config file can be loaded.");
        }
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let (config, n_args) = parse_args();
    check_nvim(&config.nvim_exe_path);

    let mut backend_functions = backend::init(&config)?;

    let mut command = backend_functions.create_command(&config);

    command.args(&n_args);

    prepare_env();
    log::debug!("Start command: {:?}", command);
    let mut child = command.spawn()?;

    backend_functions.post_start(&config, Pid::from_u32(child.id()));

    if config.fork {
        std::process::exit(0);
    } else {
        let result = child.wait()?;
        std::process::exit(result.code().unwrap_or(-1));
    }
}

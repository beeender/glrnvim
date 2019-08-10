extern crate quale;
extern crate dirs;

mod config;
mod backend;

use std::process::Command;
use quale::which;
use std::env;
use config::*;

#[cfg(target_os = "macos")]
use std::path::Path;

pub const NVIM_NAME: &str = "nvim";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_os = "macos"))]
fn prepare_env() {
}

#[cfg(target_os = "macos")]
fn prepare_env() {
    // When starting from iTerm, these env vars could cause some display issues.
    env::remove_var("TERM_PROGRAM");
    env::remove_var("TERM_PROGRAM_VERSION");
}

fn check_nvim() {
    let r = which(NVIM_NAME);
    if r == None {
        eprintln!("'nvim' executable cannot be found.");
        std::process::exit(-1);
    }
}

fn parse_args() -> (Config, Vec<String>) {
    let args: Vec<String> = env::args().collect();
    let mut n_args: Vec<String> = Vec::new();
    let mut fork: bool = true;

    for i in 1..args.len() {
        let arg = &args[i];
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
    let mut config = Config {
        fork,
        backend: String::from(""),
        fonts: vec![],
        font_size: 12 };

    let config_dir = dirs::config_dir();
    match config_dir {
        Some(mut dir) => {
            dir.push("glrnvim.conf");
            if dir.as_path().exists() {
                config::parse(dir.as_path().to_str().unwrap(), &mut config);
            }
        }
        _ => {}
    };

    return (config, n_args);
}

fn show_version() {
    let n_ver_out = Command::new("nvim")
        .arg("-v")
        .output()
        .expect("You have to install a proper nvim.");;
    if !n_ver_out.status.success()  {
        std::process::exit(-1);
    }
    println!("glrnvim {}\n", VERSION);
    println!("{}", String::from_utf8_lossy(&n_ver_out.stdout));
}

fn show_help() {
    let n_help_out = Command::new("nvim")
        .arg("-h")
        .output()
        .expect("You have to install a proper nvim.");;
    if !n_help_out.status.success()  {
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

    println!("\nConfig file: $HOME/.config/glrnvim.yml");
    println!("See https://github.com/beeender/glrnvim/blob/master/glrnvim.yml for example.");
}

fn main() {
    check_nvim();
    let (config, n_args)= parse_args();

    let mut backend_functions = backend::init(config.backend.as_str()).unwrap();
    let mut command = backend_functions.create_command(&config);

    for n_arg in &n_args {
        command.arg(n_arg);
    }

    prepare_env();
    let mut child = command.spawn().unwrap();
    if config.fork {
        std::process::exit(0);
    } else {
        let _result = child.wait().unwrap();
        std::process::exit(_result.code().unwrap_or(-1));
    }
}



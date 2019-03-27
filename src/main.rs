extern crate quale;
extern crate dirs;

use std::process::Command;
use quale::which;
use std::path::{PathBuf, Path};
use std::env;

const ALACRITTY_NAME: &str = "alacritty";
const NVIM_NAME: &str = "nvim";
const VERSION: &str = "0.1.1";

struct Config {
    fork: bool,
}

#[cfg(not(target_os = "macos"))]
fn check_alacritty() -> PathBuf {
    let r = which(ALACRITTY_NAME);
    if r == None {
        eprintln!("'alacritty' executable cannot be found.");
        std::process::exit(-1);
    }
    return r.unwrap();
}

#[cfg(target_os = "macos")]
fn check_alacritty() -> PathBuf {
    let r = which(ALACRITTY_NAME);
    if r.is_some() {
        return r.unwrap();
    }

    {
        let app_path = Path::new("/Applications/Alacritty.app/Contents/MacOS/");
        let exe_path = app_path.join(ALACRITTY_NAME);
        if exe_path.exists() && exe_path.is_file() {
            return exe_path;
        }
    }

    {
        match dirs::home_dir() {
            None => {}
            Some(home) => {
                let exe_path = home.join("Alacritty.app/Contents/MacOS/").join(ALACRITTY_NAME);
                if exe_path.exists() && exe_path.is_file() {
                    return exe_path;
                }
            }
        }
    }

    eprintln!("'alacritty' executable cannot be found.");
    std::process::exit(-1);
}

fn check_nvim() {
    let r = which(NVIM_NAME);
    if r == None {
        eprintln!("'nvim' executable cannot be found.");
        std::process::exit(-1);
    }
}

fn parse_args() -> (Option<Config>, Vec<String>) {
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
    let config = Config {fork};
    return (Some(config), n_args);
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
    let alacritty_exe = check_alacritty();
    check_nvim();
    let (config, n_args)= parse_args();
    assert!(config.is_some());
    let config_dir = dirs::config_dir();

    let mut command = Command::new(alacritty_exe);
    match config_dir {
        Some(mut dir) => {
            dir.push("glrnvim.yml");
            if dir.as_path().exists() {
                command.arg("--config-file");
                command.arg(dir.as_path().as_os_str());
            }
        }
        _ => {}
    };
    command.arg("--class");
    command.arg("glrnvim");
    command.arg("-e");
    command.arg(NVIM_NAME);

    // Enable 24-bits colors
    command.arg("+set termguicolors");
    // Set title string
    command.arg("+set title");

    for n_arg in &n_args {
        command.arg(n_arg);
    }

    let mut child = command.spawn().unwrap();
    if config.unwrap().fork {
        std::process::exit(0);
    } else {
        let _result = child.wait().unwrap();
        std::process::exit(_result.code().unwrap_or(-1));
    }
}


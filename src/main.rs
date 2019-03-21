extern crate quale;
extern crate dirs;

use std::env;
use std::process::Command;
use quale::which;

const ALACRITTY_NAME: &str = "alacritty";
const NVIM_NAME: &str = "nvim";

fn check_alacritty() {
    let r = which(ALACRITTY_NAME);
    if r == None {
        eprintln!("'alacritty' executable cannot be found.");
        std::process::exit(-1);
    }
}

fn check_nvim() {
    let r = which(NVIM_NAME);
    if r == None {
        eprintln!("'nvim' executable cannot be found.");
        std::process::exit(-1);
    }
}

fn parse_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    let mut n_args: Vec<String> = Vec::new();

    if args.len() == 1 {
        return n_args;
    }

    for i in 1..args.len() {
        let arg = &args[i];
        if arg.starts_with("-h") || arg.starts_with("--help") {
            show_help();
            std::process::exit(0);
        } else {
            n_args.push(arg.clone());
        }
    }

    return n_args;
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
        } else if line.starts_with("Options") {
            option_passed = true;
            help.push(line.to_string());
        } else if !option_passed {
            help.push(line.replace("nvim", "glrnvim"));
        } else {
            help.push(line.to_string());
        }
    }

    for line in help {
        println!("{}", line);
    }
}

fn main() {
    check_alacritty();
    check_nvim();
    let n_args = parse_args();

    let config_dir = dirs::config_dir();

    let mut command = Command::new(ALACRITTY_NAME);
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

    let result = command.spawn();
    if result.is_err() {
        std::process::exit(-1);
    }
}


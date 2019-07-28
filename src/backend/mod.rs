mod alacritty;

pub trait Functions {
    fn create_command(&self) -> std::process::Command;
}

pub fn init(backend_name: &str) -> Result<impl Functions, String> {
    if backend_name.to_lowercase() == alacritty::ALACRITTY_NAME {
        return alacritty::init();
    }

    return Err("".to_owned())
}

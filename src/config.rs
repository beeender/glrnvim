extern crate ini;
use ini::Ini;

pub struct Config {
    pub fork: bool,
    pub fonts: Vec<String>,
    pub font_size: u8
}

fn parse(path: &str, config: &mut Config) {
    let ini = Ini::load_from_file(path).unwrap();
    let section = ini.section(None::<String>).unwrap();

    if let Some(fonts) = section.get("fonts") {
        for f in fonts.split(",") {
            config.fonts.push(f.trim().to_string())
        }
    };

    if let Some(font_size) = section.get("font_size") {
        config.font_size = font_size.parse::<u8>().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use std::fs::File;
    use std::io::Write;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    struct TempConfFile {
        _dir: TempDir,
        path: String
    }


    fn make_cfg_file(content: &str) -> TempConfFile {
        // Create a directory inside of `std::env::temp_dir()`.
        let dir = tempdir().unwrap();

        let file_path = dir.path().join("glrnvim.config");
        let mut file = File::create(file_path.to_owned()).unwrap();
        file.write(content.as_bytes()).unwrap();
        file.flush().unwrap();
        drop(file);
        let tmp_conf_file = TempConfFile {_dir: dir, path: file_path.into_os_string().into_string().unwrap() };
        return tmp_conf_file;
    }

    #[test]
    fn test_parse() {
        let mut config = Config {
            fork: false,
            fonts: vec![],
            font_size: 0};

        parse(&make_cfg_file("fonts=MonoAbc ff, ac").path, &mut config);
        assert!(config.fonts[0] == "MonoAbc ff");
        assert!(config.fonts[1] == "ac");

        parse(&make_cfg_file("font_size=15").path, &mut config);
        assert!(config.font_size == 15);

        parse(&make_cfg_file("font_size=sadfa").path, &mut config);
        assert!(config.font_size == 15);
    }
}

use std::{path::{Path, PathBuf}, fs};

use serde_json::Value;

pub fn find_chrome_pak<P>(dir: P) -> Option<PathBuf>
    where P: AsRef<Path>{
    
    for file in dir.as_ref().read_dir().unwrap() {
        let file = file.unwrap();
        let typ = file.file_type().unwrap();

        if typ.is_file() {
            if file.file_name() == "chrome_100_percent.pak" {
                return Some(file.path());
            }
        } else if typ.is_dir() {
            match find_chrome_pak(file.path()) {
                Some(x) => return Some(x),
                None => {},
            }
        }
    }

    None
}

pub fn get_flatpak_dir(id: &str) -> Option<PathBuf> {
    let base = PathBuf::from(format!("/var/lib/flatpak/app/{}/current/active/files/", id));
    if !base.exists() {
        return None;
    }

    let chrome_pak = find_chrome_pak(&base).expect("Can't find Electron app.");
    let dir = chrome_pak.parent().unwrap();
    
    Some(PathBuf::from(dir))
}

pub fn get_flatpak_replaced_file(id: &str) -> PathBuf {
    let base = PathBuf::from(format!("/var/lib/flatpak/app/{}/current/active/files/", id));
    let manifest_text = fs::read_to_string(base.join("manifest.json")).unwrap();
    let manifest: Value = serde_json::from_str(&manifest_text).unwrap();

    let command = manifest.get("command").unwrap().as_str().unwrap();
    let wrapper = fs::read_to_string(base.join("bin/").join(command)).unwrap();
    let mut tok = wrapper.split(" ");

    while tok.next().unwrap() != "zypak-wrapper" {}
    let replaced_relative = Path::new(tok.next().unwrap()).strip_prefix("/app/").unwrap();

    base.join(replaced_relative)
}

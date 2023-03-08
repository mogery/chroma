use std::{path::{PathBuf, Path}, fs};

use sha3::{Sha3_256, Digest};

pub fn get_chroma_dir() -> PathBuf {
    let chroma_dir = Path::new("/var/lib/chroma/");
    if !chroma_dir.exists() {
        println!("Creating chroma directory at {:?}...", chroma_dir);
        fs::create_dir(&chroma_dir).expect("Failed to create directory .chroma");
    }

    chroma_dir.to_owned()
}

pub fn get_apps_dir<P>(chroma_dir: P) -> PathBuf
    where P: AsRef<Path>
{
    let apps_dir = chroma_dir.as_ref().join("apps");
    if !apps_dir.exists() {
        fs::create_dir(&apps_dir).expect("Failed to create directory .chroma/apps");
    }

    apps_dir
}

pub fn get_electron_dir<P>(chroma_dir: P) -> Option<PathBuf>
    where P: AsRef<Path>
{
    let electron_dir = chroma_dir.as_ref().join("electron");

    if electron_dir.exists() {
        Some(electron_dir)
    } else {
        None
    }
}

fn get_hash<T>(x: T) -> String
    where T: AsRef<str> {
    let mut hasher = Sha3_256::new();
    hasher.update(x.as_ref());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn get_path_hash<P>(path: P) -> String
    where P: AsRef<Path> {
    get_hash(path.as_ref().to_str().unwrap())
}

pub fn get_flatpak_hash(id: &str) -> String {
    get_hash("flatpak:".to_owned() + id)
}
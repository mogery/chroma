use std::{fs::{self, hard_link, File}, path::{Path, PathBuf}, os::unix::fs::symlink, f32::consts::E, process};
use clap::{Parser, Subcommand};
use sha3::{Sha3_256, Digest};
use serde_json::Value;

#[derive(Subcommand)]
enum Commands {
    Flatpak {
        id: String,
    },
    Raw {
        path: String,
    },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn get_chroma_dir() -> PathBuf {
    let chroma_dir = Path::new("/var/lib/chroma/");
    if !chroma_dir.exists() {
        println!("Creating chroma directory at {:?}...", chroma_dir);
        fs::create_dir(&chroma_dir).expect("Failed to create directory .chroma");
    }

    chroma_dir.to_owned()
}

fn get_apps_dir<P>(chroma_dir: P) -> PathBuf
    where P: AsRef<Path>
{
    let apps_dir = chroma_dir.as_ref().join("apps");
    if !apps_dir.exists() {
        fs::create_dir(&apps_dir).expect("Failed to create directory .chroma/apps");
    }

    apps_dir
}

fn get_electron_dir<P>(chroma_dir: P) -> Option<PathBuf>
    where P: AsRef<Path>
{
    let electron_dir = chroma_dir.as_ref().join("electron");

    if electron_dir.exists() {
        Some(electron_dir)
    } else {
        None
    }
}

fn get_path_hash<P>(path: P) -> String
    where P: AsRef<Path> {
    let pstr = path.as_ref().to_str().unwrap();
    let mut hasher = Sha3_256::new();
    hasher.update(pstr);
    let result = hasher.finalize();
    hex::encode(result)
}

fn get_flatpak_hash(id: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update("flatpak:".to_owned() + id);
    let result = hasher.finalize();
    hex::encode(result)
}

fn find_chrome_pak<P>(dir: P) -> Option<PathBuf>
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

fn get_flatpak_dir(id: &str) -> Option<PathBuf> {
    let base = PathBuf::from(format!("/var/lib/flatpak/app/{}/current/active/files/", id));
    if !base.exists() {
        return None;
    }

    let chrome_pak = find_chrome_pak(&base).expect("Can't find Electron app.");
    let dir = chrome_pak.parent().unwrap();
    
    Some(PathBuf::from(dir))
}

fn get_flatpak_replaced_file(id: &str) -> PathBuf {
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

fn main() {
    let args = Args::parse();

    let chroma_dir = get_chroma_dir();
    let electron_dir = get_electron_dir(&chroma_dir).expect("Electron not found");
    let apps_dir = get_apps_dir(&chroma_dir);

    let (dir_path, hash, target_path) = match &args.command {
        Commands::Flatpak { id } => {
            let dir_path = get_flatpak_dir(&id).expect("Flatpak app doesn't exist");
            let hash = get_flatpak_hash(&id);
            let target_path = get_flatpak_replaced_file(&id);
            (dir_path, hash, target_path)
        },
        Commands::Raw { path } => {
            let target_path = Path::new(&path).canonicalize().expect("Invalid path");
            let dir_path = target_path.parent().unwrap().to_owned();
            let hash = get_path_hash(&target_path);
            (dir_path, hash, target_path)
        },
    };

    println!("{:?} {:?} {:?}", dir_path, hash, target_path);

    let app_dir = apps_dir.join(Path::new(&hash));
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir).unwrap();
    }

    fs::create_dir(&app_dir).expect(&format!("Failed to create app directory {:?}", app_dir));

    for file in electron_dir.read_dir().unwrap() {
        let file = file.unwrap();
        let file_path = file.path();
        let filename = file_path.file_name().unwrap();
        let typ = file.file_type().unwrap();
        if typ.is_dir() {
            if filename != "resources" {
                symlink(file.path(), app_dir.join(file.path().file_name().unwrap())).unwrap();
            }
        } else if typ.is_file() {
            if filename == "electron" {
                hard_link(file.path(), app_dir.join(target_path.file_name().unwrap())).unwrap();
            } else {
                hard_link(file.path(), app_dir.join(file.path().file_name().unwrap())).unwrap();
            }
        } else {
            panic!("{:?} is neither a directory or a file?", file.path());
        }
    }

    symlink(dir_path.join("resources"), app_dir.join("resources")).unwrap();

    for file in dir_path.read_dir().unwrap() {
        let file = file.unwrap();
        let file_path = file.path();
        let filename = file_path.file_name().unwrap();
        let typ = file.file_type().unwrap();

        if typ.is_dir() {
            if filename != "resources" {
                fs::remove_dir_all(file.path()).unwrap();
            }
        } else {
            fs::remove_file(file.path()).unwrap();
        }
    }

    let _ = fs::remove_file(&target_path).is_ok();

    symlink(app_dir.join(target_path.file_name().unwrap()), &target_path).unwrap();

    if let Commands::Flatpak { id } = args.command {
        let x = process::Command::new("flatpak")
            .args(["override", &format!("--filesystem={}", app_dir.to_str().unwrap()), &id])
            .output()
            .expect("Failed to do Flatpak sandboxing");
        
        println!("Flatpak did a {}: {:?} {:?}", x.status, std::str::from_utf8(&x.stdout), std::str::from_utf8(&x.stderr))
    }

    println!("{:#?}", app_dir);
}

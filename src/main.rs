use std::{fs::{self, hard_link}, path::Path, os::unix::fs::symlink, process};
use clap::{Parser, Subcommand};

use crate::{structure::{get_chroma_dir, get_electron_dir, get_apps_dir, get_flatpak_hash, get_path_hash}, flatpak::{get_flatpak_dir, get_flatpak_replaced_file}};

pub(crate) mod structure;
pub(crate) mod flatpak;

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

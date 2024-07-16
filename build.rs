use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn get_commit_hash() -> String {
    let child = Command::new("git").args(&["describe", "--always"]).output();
    match child {
        Ok(child) => {
            let buf = String::from_utf8(child.stdout).expect("failed to read stdout");
            buf.to_string()
        }
        Err(err) => {
            eprintln!("`git describe` err: {}", err);
            "".to_string()
        }
    }
}

fn main() {
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap().to_string();
    let commit_hash = get_commit_hash();
    let version = "rust-openbmclapi/".to_string() + &pkg_version + "-" + &commit_hash;
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("PKG_VERSION");
    fs::write(&dest_path, pkg_version).unwrap();
    let dest_path = Path::new(&out_dir).join("VERSION");
    fs::write(&dest_path, version).unwrap();
}

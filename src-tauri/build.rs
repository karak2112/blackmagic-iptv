use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    tauri_build::build();
    prepare_mpv_windows();
}

fn prepare_mpv_windows() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("..");
    let src_dir = root.join("third_party/mpv/win");
    let dll = src_dir.join("libmpv-2.dll");
    let import_lib = src_dir.join("mpv.lib");

    if !dll.exists() {
        println!(
            "cargo:warning=libmpv-2.dll missing at {} - run npm run fetch-mpv",
            src_dir.display()
        );
        return;
    }

    if !import_lib.exists() {
        let script = root.join("scripts/generate-mpv-lib.ps1");
        if script.exists() {
            let status = Command::new("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    script.to_str().unwrap(),
                    "-Dest",
                    src_dir.to_str().unwrap(),
                ])
                .status();

            match status {
                Ok(s) if s.success() && import_lib.exists() => {}
                _ => println!(
                    "cargo:warning=mpv.lib missing - run: npm run fetch-mpv (or scripts/generate-mpv-lib.ps1)"
                ),
            }
        }
    }

    if import_lib.exists() {
        println!("cargo:rustc-link-search=native={}", src_dir.display());
        println!("cargo:rerun-if-changed={}", import_lib.display());
    }

    println!("cargo:rerun-if-changed={}", dll.display());

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = workspace_target_dir(&manifest_dir, &profile);

    if let Err(e) = fs::create_dir_all(&target_dir) {
        println!("cargo:warning=failed to create target dir: {e}");
        return;
    }

    for entry in fs::read_dir(&src_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("dll") {
            continue;
        }
        let dest = target_dir.join(path.file_name().unwrap());
        if let Err(e) = fs::copy(&path, &dest) {
            println!("cargo:warning=failed to copy {}: {e}", path.display());
        }
    }
}

fn workspace_target_dir(manifest_dir: &Path, profile: &str) -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(dir).join(profile);
    }
    manifest_dir.join("../../target").join(profile)
}

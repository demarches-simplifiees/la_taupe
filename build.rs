// build.rs
use std::env;
use std::path::Path;
use std::process::Command;

/*
build script for the project
*/
fn main() {
    // taken from https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let profile = env::var("PROFILE").unwrap_or_default();

    if profile == "release" {
        download_models_if_needed();
    } else {
        fake_download_models();
    }
}

fn fake_download_models() {
    let models_dir = Path::new("models");
    let detection_model = models_dir.join("text-detection.rten");
    let recognition_model = models_dir.join("text-recognition.rten");

    // on cree les fichiers models vides si les fichiers n'existent pas
    if !detection_model.exists() {
        std::fs::File::create(&detection_model).expect("Failed to create detection model file");
    }

    if !recognition_model.exists() {
        std::fs::File::create(&recognition_model).expect("Failed to create recognition model file");
    }
}

fn download_models_if_needed() {
    let models_dir = Path::new("models");
    let detection_model = models_dir.join("text-detection.rten");
    let recognition_model = models_dir.join("text-recognition.rten");

    let missing_detection_model =
        !detection_model.exists() || detection_model.metadata().unwrap().len() == 0;
    let missing_recognition_model =
        !recognition_model.exists() || recognition_model.metadata().unwrap().len() == 0;

    if missing_detection_model || missing_recognition_model {
        println!("Downloading models...");

        let output = Command::new("bash")
            .arg("download-models.sh")
            .current_dir(".")
            .output()
            .expect("Failed to execute download-model.sh. Make sure bash is available and the script exists.");

        if !output.status.success() {
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
            panic!(
                "download-model.sh failed with exit code: {:?}",
                output.status.code()
            );
        }

        println!("Models downloaded successfully");
    } else {
        println!("Models already exist and are not empty, skipping download");
    }
}

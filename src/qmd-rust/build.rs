use std::process::Command;

fn main() {
    // Check if sqlite-vec is available
    let output = Command::new("pkg-config")
        .args(["--exists", "sqlite3"])
        .output();

    match output {
        Ok(_) => {
            println!("sqlite3 found via pkg-config");
        }
        Err(_) => {
            println!("Note: Using rusqlite with bundled SQLite");
        }
    }

    // Configure OpenMP linking for llama-cpp on macOS
    #[cfg(target_os = "macos")]
    {
        // Always add OpenMP paths when building on macOS
        // This is needed for llama-cpp feature
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/libomp/lib");
        println!("cargo:rustc-link-lib=dylib=omp");
    }
}

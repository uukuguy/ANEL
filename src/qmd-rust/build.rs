use std::path::Path;
use std::process::{Command, Stdio};

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
}

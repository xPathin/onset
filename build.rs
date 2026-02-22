use std::process::Command;

fn main() {
    let version = Command::new("git")
        .args(["describe", "--long", "--tags", "--abbrev=7"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| {
            let s = s.trim().trim_start_matches('v');
            // Format: <tag>-<n>-g<hash>
            // If n=0 we're on a tagged release, show just the version
            if s.contains("-0-g") {
                s.split("-0-g").next().unwrap().to_string()
            } else {
                s.to_string()
            }
        });

    println!(
        "cargo:rustc-env=ONSET_VERSION={}",
        version.unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap())
    );
}

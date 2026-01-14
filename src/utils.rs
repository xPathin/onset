use std::path::Path;

pub fn binary_exists(binary: &str) -> bool {
    if binary.starts_with('/') {
        Path::new(binary).exists()
    } else {
        std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(binary).exists()))
            .unwrap_or(false)
    }
}

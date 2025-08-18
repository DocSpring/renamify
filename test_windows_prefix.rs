use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Write a file
    fs::write(&test_file, "hello world").unwrap();
    
    // Get canonical path (which may add \\?\ prefix)
    let canonical = test_file.canonicalize().unwrap();
    
    println!("Original path: {}", test_file.display());
    println!("Canonical path: {}", canonical.display());
    
    let path_str = canonical.to_string_lossy();
    println!("Path string: {:?}", path_str);
    
    // Check different patterns
    if path_str.starts_with(r"\\?\") {
        println!("Starts with r\"\\\\?\\\"");
    }
    if path_str.starts_with("\\\\?\\") {
        println!("Starts with \"\\\\\\\\?\\\\\"");
    }
    
    // Check individual characters
    let chars: Vec<char> = path_str.chars().take(4).collect();
    println!("First 4 chars: {:?}", chars);
}
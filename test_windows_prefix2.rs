use std::fs;
use std::path::PathBuf;

fn main() {
    let test_file = PathBuf::from(r"C:\Users\Nathan\AppData\Local\Temp\test123\test.txt");
    
    // Create the directory
    let _ = fs::create_dir_all(r"C:\Users\Nathan\AppData\Local\Temp\test123");
    
    // Write a file
    fs::write(&test_file, "hello world").unwrap();
    
    // Get canonical path (which may add \\?\ prefix)
    let canonical = test_file.canonicalize().unwrap();
    
    println!("Original path: {}", test_file.display());
    println!("Canonical path: {}", canonical.display());
    
    let path_str = canonical.to_string_lossy();
    println!("Path string: {:?}", path_str.to_string());
    
    // Check different patterns
    if path_str.starts_with(r"\\?\") {
        println!("Starts with r\"\\\\?\\\"");
    }
    if path_str.starts_with("\\\\?\\") {
        println!("Starts with \"\\\\\\\\?\\\\\"");
    }
    
    // Check individual characters
    let chars: Vec<char> = path_str.chars().take(8).collect();
    println!("First 8 chars: {:?}", chars);
    
    // Check bytes
    let bytes: Vec<u8> = path_str.bytes().take(8).collect();
    println!("First 8 bytes: {:?}", bytes);
}
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;
use tempfile::TempDir;

use storage_analyzer_lib::disk_utils::{get_disk_space, list_files};

#[test]
fn test_get_disk_space() {
    let dir = env::current_dir();
    assert!(dir.is_ok());

    let path = dir.unwrap();
    let result = get_disk_space(&path);

    assert!(result.is_some());
    let (total, free) = result.unwrap();
    assert!(total > 0);
    assert!(free > 0);
}

#[test]
fn test_list_files() -> Result<(), Box<dyn Error>> {
    // Create a temporary directory structure
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create a subdirectory
    let sub_dir = temp_path.join("subdir");
    fs::create_dir(&sub_dir)?;

    // Create some test files
    let test_file1 = temp_path.join("test1.txt");
    let file1 = File::create(&test_file1)?;

    let test_file2 = sub_dir.join("test2.txt");
    let file2 = File::create(&test_file2)?;

    // Test the list_files function
    let files = list_files(temp_path).unwrap();
    let file_info1 = files.last().unwrap();
    let file_info2 = files.first().unwrap();

    // Verify that
    // temppath
    // ├── test1.txt
    // ├── subdir
    // │   ├── test2.txt
    assert_eq!(files.len(), 2);
    assert_eq!(file_info1.size, file1.metadata()?.len());
    assert_eq!(file_info1.directory, temp_path.display().to_string());
    assert_eq!(file_info2.size, file2.metadata()?.len());
    assert_eq!(file_info2.directory, sub_dir.display().to_string());

    Ok(())
}

#[test]
fn test_get_disk_space_invalid_path() {
    let invalid_path = Path::new("/nonexistent/path");
    let result = get_disk_space(invalid_path);
    assert!(result.is_none());
}

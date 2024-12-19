use std::time::SystemTime;
use storage_analyzer_lib::disk_utils::{FileInfo, FileType};
use storage_analyzer_lib::size_analysis::{
    analyze_file_categories, analyze_largest_dirs, analyze_largest_files,
};

fn get_test_files() -> Vec<FileInfo> {
    vec![
        FileInfo {
            name: "file1.jpg".to_string(),
            path: "/path1/file1.jpg".to_string(),
            directory: "/path1".to_string(),
            file_type: FileType::Photos,
            size: 5 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
        FileInfo {
            name: "file2.mp3".to_string(),
            path: "/path1/file2.mp3".to_string(),
            directory: "/path1".to_string(),
            file_type: FileType::Audio,
            size: 10 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
        FileInfo {
            name: "file3.txt".to_string(),
            path: "/path2/file3.txt".to_string(),
            directory: "/path2".to_string(),
            file_type: FileType::Documents,
            size: 15 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
        FileInfo {
            name: "file4.mov".to_string(),
            path: "/path2/file4.mov".to_string(),
            directory: "/path2".to_string(),
            file_type: FileType::Videos,
            size: 20 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
        FileInfo {
            name: "file5.zip".to_string(),
            path: "/path3/file5.zip".to_string(),
            directory: "/path3".to_string(),
            file_type: FileType::Archives,
            size: 25 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
        FileInfo {
            name: "file6.exe".to_string(),
            path: "/path3/file6.exe".to_string(),
            directory: "/path3".to_string(),
            file_type: FileType::Executables,
            size: 30 * 1024 * 1024,
            created: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
        },
    ]
}

#[test]
fn test_analyze_file_categories() {
    let files = get_test_files();
    let categories = analyze_file_categories(&files);

    assert_eq!(*categories.get(&FileType::Photos).unwrap(), 5 * 1024 * 1024);
    assert_eq!(*categories.get(&FileType::Audio).unwrap(), 10 * 1024 * 1024);
    assert_eq!(
        *categories.get(&FileType::Documents).unwrap(),
        15 * 1024 * 1024
    );
    assert_eq!(
        *categories.get(&FileType::Videos).unwrap(),
        20 * 1024 * 1024
    );
    assert_eq!(
        *categories.get(&FileType::Archives).unwrap(),
        25 * 1024 * 1024
    );
    assert_eq!(
        *categories.get(&FileType::Executables).unwrap(),
        30 * 1024 * 1024
    );
}

#[test]
fn test_analyze_largest_files() {
    let files = get_test_files();
    let largest_files = analyze_largest_files(&files, 3);

    assert_eq!(largest_files.len(), 3);
    assert_eq!(largest_files[0].name, "file6.exe");
    assert_eq!(largest_files[1].name, "file5.zip");
    assert_eq!(largest_files[2].name, "file4.mov");
}

#[test]
fn test_analyze_largest_dirs() {
    let files = get_test_files();
    let largest_dirs = analyze_largest_dirs(&files, 2);

    let (size1, dir1) = largest_dirs.first().unwrap();
    let (size2, dir2) = largest_dirs.last().unwrap();
    assert_eq!(*size1, 55 * 1024 * 1024);
    assert_eq!(dir1, "/path3");
    assert_eq!(*size2, 35 * 1024 * 1024);
    assert_eq!(dir2, "/path2");
}

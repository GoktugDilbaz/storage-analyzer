use rayon::prelude::*;
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;
use std::vec::Vec;
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum FileType {
    Photos,
    Videos,
    Audio,
    Documents,
    Archives,
    Executables,
    Other,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub directory: String,
    pub file_type: FileType,
    pub size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
}

pub fn get_drives() -> Vec<String> {
    let system = System::new_all();
    system
        .disks()
        .iter()
        .map(|disk| disk.mount_point().display().to_string())
        .collect()
}

pub fn get_disk_space(path: &Path) -> Option<(u64, u64)> {
    let mut sys = System::new_all();
    sys.refresh_disks_list();

    sys.disks()
        .iter()
        .find(|disk| path.starts_with(disk.mount_point()))
        .map(|disk| (disk.total_space(), disk.available_space()))
}

pub fn list_files(dir: &Path) -> Result<Vec<FileInfo>, io::Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("Warning: Permission denied for directory: {:?}", dir);
            return Ok(vec![]);
        }
        Err(e) => return Err(e),
    };

    let results: Result<Vec<_>, io::Error> = entries
        .par_iter()
        .map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                Ok(list_files(&path)?)
            } else if metadata.is_file() {
                Ok(vec![create_file_info(&metadata, path.as_path())])
            } else {
                eprintln!("Warning: Skipping entry {:?}", path);
                Ok(vec![])
            }
        })
        .collect();

    match results {
        Ok(nested_results) => Ok(nested_results.into_iter().flatten().collect()),
        Err(e) => Err(e),
    }
}

fn create_file_info(metadata: &fs::Metadata, path: &Path) -> FileInfo {
    FileInfo {
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        path: path.display().to_string(),
        directory: path.parent().unwrap_or(Path::new("")).display().to_string(),
        file_type: categorize_file(path),
        size: metadata.len(),
        created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    }
}

fn categorize_file(path: &Path) -> FileType {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "raw" | "heic" => FileType::Photos,

        "mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" => FileType::Videos,

        "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => FileType::Audio,

        "pdf" | "doc" | "docx" | "txt" | "rtf" | "xls" | "xlsx" | "ppt" | "pptx" | "odt"
        | "ods" | "odp" | "csv" => FileType::Documents,

        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => FileType::Archives,

        "exe" | "msi" | "dll" | "app" | "dmg" | "deb" | "rpm" => FileType::Executables,

        _ => FileType::Other,
    }
}

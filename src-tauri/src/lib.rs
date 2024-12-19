pub mod disk_utils;
pub mod size_analysis;

const PIE_CHART_SLICE_COUNT: usize = 20;

#[tauri::command]
async fn get_drives() -> Vec<String> {
    disk_utils::get_drives()
}

#[tauri::command]
async fn analyze_drive(path: String) -> Result<DriveAnalysis, String> {
    let path = std::path::Path::new(&path);
    let mut analysis = DriveAnalysis::default();

    if let Some((total, free)) = disk_utils::get_disk_space(path) {
        let used = total - free;
        analysis.total = total;
        analysis.used = used;
        analysis.categories.other = used;
        analysis.categories.free = free;
    } else {
        return Err(format!("Error getting disk space for: {}", path.display()));
    }

    let files_result = disk_utils::list_files(path);
    if files_result.is_err() {
        return Err(format!("Error getting files for: {}", path.display()));
    }
    let files = files_result.unwrap();

    for (category, size) in size_analysis::analyze_file_categories(&files) {
        match category {
            disk_utils::FileType::Photos => analysis.categories.photos = size,
            disk_utils::FileType::Videos => analysis.categories.videos = size,
            disk_utils::FileType::Audio => analysis.categories.audio = size,
            disk_utils::FileType::Executables => analysis.categories.executables = size,
            disk_utils::FileType::Documents => analysis.categories.documents = size,
            disk_utils::FileType::Archives => analysis.categories.archives = size,
            disk_utils::FileType::Other => analysis.categories.other = size,
        }
    }

    for info in size_analysis::analyze_largest_files(&files, PIE_CHART_SLICE_COUNT) {
        analysis.largest_files.push(File::from(info));
    }

    for (size, path) in size_analysis::analyze_largest_dirs(&files, PIE_CHART_SLICE_COUNT) {
        analysis.largest_directories.push(Directory { path, size });
    }

    Ok(analysis)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_drives, analyze_drive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize)]
struct DriveAnalysis {
    total: u64,
    used: u64,
    categories: Categories,
    largest_files: Vec<File>,
    largest_directories: Vec<Directory>,
}

impl Default for DriveAnalysis {
    fn default() -> Self {
        DriveAnalysis {
            total: 0,
            used: 0,
            categories: Categories {
                photos: 0,
                videos: 0,
                audio: 0,
                executables: 0,
                documents: 0,
                archives: 0,
                other: 0,
                free: 0,
            },
            largest_files: vec![],
            largest_directories: vec![],
        }
    }
}

#[derive(serde::Serialize)]
struct Categories {
    photos: u64,
    videos: u64,
    audio: u64,
    executables: u64,
    documents: u64,
    archives: u64,
    other: u64,
    free: u64,
}

#[derive(serde::Serialize)]
struct File {
    name: String,
    path: String,
    file_type: String,
    size: u64,
    created: std::time::SystemTime,
    modified: std::time::SystemTime,
}

#[derive(serde::Serialize)]
struct Directory {
    path: String,
    size: u64,
}

impl File {
    fn from(info: disk_utils::FileInfo) -> File {
        File {
            name: info.name,
            path: info.path,
            file_type: match info.file_type {
                disk_utils::FileType::Photos => "Photo".to_string(),
                disk_utils::FileType::Videos => "Video".to_string(),
                disk_utils::FileType::Audio => "Audio".to_string(),
                disk_utils::FileType::Documents => "Document".to_string(),
                disk_utils::FileType::Archives => "Archive".to_string(),
                disk_utils::FileType::Executables => "Executable".to_string(),
                disk_utils::FileType::Other => "Other".to_string(),
            },
            size: info.size,
            created: info.created,
            modified: info.modified,
        }
    }
}

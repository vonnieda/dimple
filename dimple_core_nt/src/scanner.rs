use media_file::ScannedFile;
use walkdir::WalkDir;

pub mod media_file;

pub struct Scanner {
}

impl Scanner {
    pub fn scan_directory(directory_path: &str) -> Vec<media_file::ScannedFile> {
        WalkDir::new(directory_path).into_iter()
            .filter(|dir_entry| dir_entry.is_ok())
            .map(|dir_entry| dir_entry.unwrap())
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .map(|file_entry| ScannedFile::new(file_entry.path().to_str().unwrap()))
            .filter_map(|mf| mf.ok())
            .collect()
    }
}

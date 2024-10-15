use media_file::MediaFile;
use walkdir::WalkDir;

pub mod media_file;

pub struct Scanner {
}

impl Scanner {
    pub fn scan_directory(directory_path: &str) -> Vec<media_file::MediaFile> {
        WalkDir::new(directory_path).into_iter()
            .filter(|dir_entry| dir_entry.is_ok())
            .map(|dir_entry| dir_entry.unwrap())
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .map(|file_entry| MediaFile::new(file_entry.path().to_str().unwrap()))
            .collect()
    }
}

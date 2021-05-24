use std::fs::{File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::app::settings::{FileSetting, Setting};

impl FileSetting for ImagesDirectorySetting {
    type FileResult = ReadDir;
    type Error = std::io::Error;

    fn read_file(&self) -> Result<Self::FileResult, Self::Error> {
        Path::new(self.images_dir.as_str()).read_dir()
    }

    fn backing_path_mut(&mut self) -> &mut String {
        &mut self.images_dir
    }
}

impl Setting for ImagesDirectorySetting {
    const SETTING_NAME: &'static str = "Images Directory";

    fn is_valid(&self) -> bool {
        Path::new(self.images_dir.as_str()).is_dir()
    }

    fn display_validity(&self) -> String {
        let path = Path::new(self.images_dir.as_str());
        String::from(if !path.exists() {
            "directory does not exist"
        } else if !path.is_dir() {
            "file is not a directory"
        } else {
            "✅"
        })
    }
}

impl FileSetting for NamesFileSetting {
    type FileResult = Vec<String>;
    type Error = std::io::Error;

    fn read_file(&self) -> Result<Self::FileResult, Self::Error> {
        let path = Path::new(self.names_file.as_str());
        let file = File::open(path)?;
        BufReader::new(file).lines().collect()
    }

    fn backing_path_mut(&mut self) -> &mut String {
        &mut self.names_file
    }
}

impl Setting for NamesFileSetting {
    const SETTING_NAME: &'static str = "Names File";

    fn is_valid(&self) -> bool {
        self.read_file().is_ok()
    }

    fn display_validity(&self) -> String {
        let path = Path::new(self.names_file.as_str());
        String::from(if !path.exists() {
            "path does not exist"
        } else if let Some(ext) = path.extension() {
            if ext != "names" {
                "path does not end with .names"
            } else {
                "✅"
            }
        } else if path.is_dir() {
            "path is a directory"
        } else {
            "path does not have an extension"
        })
    }
}

#[derive(Default)]
pub struct ImagesDirectorySetting {
    pub images_dir: String,
}

#[derive(Default)]
pub struct NamesFileSetting {
    pub names_file: String,
}

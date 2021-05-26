use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{PathBuf};

use image::{DynamicImage, ImageError};

static SUPPORTED_IMAGE_TYPES: [&str; 3] = ["jpg", "JPG", "JPEG"];

#[derive(Debug)]
pub struct ImageFile(PathBuf);

#[derive(Debug)]
pub enum ImageFileError {
    NotAFile,
    NotAnImage,
}

impl TryFrom<DirEntry> for ImageFile {
    type Error = ImageFileError;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        ImageFile::new(value.path())
    }
}

impl ImageFile {
    pub fn as_image(&self) -> Result<DynamicImage, ImageError> {
        match image::open(self.0.as_path()) {
            Ok(img) => Ok(img),
            Err(err) => Err(err),
        }
    }

    pub fn as_path(&self) -> PathBuf {
        self.0.clone()
    }

    pub(crate) fn new(entry: PathBuf) -> Result<ImageFile, ImageFileError> {
        let is_supported_image_type = entry
            .as_path()
            .extension()
            .map(|ext| {
                SUPPORTED_IMAGE_TYPES
                    .iter()
                    .map(|str| OsStr::new(str))
                    .any(|x| x == ext)
            })
            .unwrap_or_default();
        if !entry.is_file() {
            Err(ImageFileError::NotAFile)
        } else if !is_supported_image_type {
            Err(ImageFileError::NotAnImage)
        } else {
            Ok(ImageFile(entry))
        }
    }
}

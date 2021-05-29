use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::PathBuf;

use image::{DynamicImage, ImageError};

use crate::app::bbox::{BBox};

static SUPPORTED_IMAGE_TYPES: [&str; 3] = ["jpg", "JPG", "JPEG"];

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
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
    pub fn load_labels(&self) -> Vec<BBox> {
        let parent = match self.0.parent() {
            None => panic!("oh god oh fuck where is the file"),
            Some(p) => p.to_str().unwrap(),
        };
        let txt_path = match self.0.file_stem().and_then(|stem| stem.to_str()) {
            None => {
                println!("heck");
                return Vec::new();
            }
            Some(stem) => format!("{}/{}.txt", parent, stem),
        };
        match File::open(txt_path) {
            Ok(f) =>
                BufReader::new(f)
                    .lines()
                    .map(|r_line| r_line.expect("invalid line read"))
                    .filter(|line| !line.is_empty())
                    .map(|line| BBox::try_from(line.as_str()))
                    .map(|r_bbox| r_bbox.expect("box failed to parse"))
                    .collect::<Vec<BBox>>(),
            Err(_) => {
                println!("heck");
                Vec::new()
            }
        }
    }

    pub fn save_labels(&self, labels: &[BBox]) -> std::io::Result<()> {
        let parent = match self.0.parent() {
            None => panic!("oh god oh fuck where is the file"),
            Some(p) => p.to_str().unwrap(),
        };
        let txt_path = match self.0.file_stem().and_then(|stem| stem.to_str()) {
            None => panic!("oh god oh fuck where is the file"),
            Some(stem) => format!("{}/{}.txt", parent, stem),
        };
        let f = File::with_options()
            .create(true)
            .write(true)
            .open(txt_path)?;
        let lw = &mut LineWriter::new(f);
        for bbox in labels {
            lw.write_all(bbox.yolo_format().as_bytes())?;
            lw.write_all(b"\n")?;
        }
        Ok(())
    }

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

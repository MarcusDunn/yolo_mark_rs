use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageError};

use crate::app::bbox::BBox;

static SUPPORTED_IMAGE_TYPES: [&str; 3] = ["jpg", "JPG", "JPEG"];

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ImageFile(PathBuf);

#[derive(Debug)]
pub enum Error {
    NotAFile,
    NotAnImage,
}

impl TryFrom<DirEntry> for ImageFile {
    type Error = Error;

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
        let txt_path = match self.0.file_stem().and_then(OsStr::to_str) {
            None => {
                println!("could not get file stem of {:?}", self.0);
                return Vec::new();
            }
            Some(stem) => format!("{}/{}.txt", parent, stem),
        };
        match File::open(&txt_path) {
            Ok(f) => BufReader::new(f)
                .lines()
                .map(|r_line| r_line.expect("invalid line read"))
                .filter(|line| !line.is_empty())
                .map(|line| BBox::try_from(line.as_str()))
                .filter_map(|r_bbox| match r_bbox {
                    Err(err) => {
                        println!(
                            "WARNING: error when parsing boxes from file {} {}",
                            txt_path, err
                        );
                        println!("ignoring for now . . . ");
                        None
                    }
                    Ok(bbox) => Some(bbox),
                })
                .collect(),
            Err(err) => {
                println!("could not open {}: {}", txt_path, err);
                Vec::new()
            }
        }
    }

    pub fn save_labels(&self, labels: &[BBox]) -> std::io::Result<()> {
        let parent = self.0.parent().map_or(Some(""), Path::to_str);
        let parent = match parent {
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("{:?} was not valid unicode", parent),
                ))
            }
            Some(str) => str,
        };
        let stem = match self.0.file_stem().and_then(OsStr::to_str) {
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("could not create stem from {:?}", self),
                ))
            }
            Some(stem) => stem,
        };
        let txt_path = format!("{}/{}.txt", parent, stem);
        let f = File::with_options()
            .create(true)
            .write(true)
            .open(&txt_path)?;
        let result = f.set_len(0);
        BufWriter::new(f).write_all(Self::labels_to_string(labels).into_bytes().as_slice())?;
        // we evaluate the result after writing so we don't exit without writing SOMETHING to the
        // file even if it has garbage left over at the end
        result?;
        Ok(())
    }

    fn labels_to_string(labels: &[BBox]) -> String {
        labels
            .iter()
            .map(BBox::yolo_format)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn as_image(&self) -> Result<DynamicImage, ImageError> {
        image::open(self.0.as_path())
    }

    pub fn as_path(&self) -> PathBuf {
        self.0.clone()
    }

    pub(crate) fn new(entry: PathBuf) -> Result<ImageFile, Error> {
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
            Err(Error::NotAFile)
        } else if is_supported_image_type {
            Ok(ImageFile(entry))
        } else {
            Err(Error::NotAnImage)
        }
    }
}

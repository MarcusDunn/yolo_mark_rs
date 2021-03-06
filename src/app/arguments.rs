use core::fmt;
use std::env::Args;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{File, ReadDir};
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use crate::app::images::Images;

pub enum ArgumentError {
    InvalidNumber(String),
    FileDoesNotExist(String),
    InvalidFileType(String),
    ReadError(String),
}

impl From<ParseIntError> for ArgumentError {
    fn from(err: ParseIntError) -> Self {
        ArgumentError::InvalidNumber(format!("{}", err))
    }
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (err, explanation) = match self {
            ArgumentError::InvalidNumber(explanation) => ("InvalidNumber", explanation),
            ArgumentError::FileDoesNotExist(explanation) => ("FileDoesNotExist", explanation),
            ArgumentError::InvalidFileType(explanation) => ("InvalidFileType", explanation),
            ArgumentError::ReadError(explanation) => ("ReadError", explanation),
        };
        write!(f, "{}: {}", err, explanation)
    }
}

pub struct Arguments {
    pub image_dir: Images,
    pub names: Vec<String>,
    pub names_dir: PathBuf,
}

impl Arguments {
    fn new(image_dir: ReadDir, names: Vec<String>, names_dir: PathBuf) -> Arguments {
        let images = image_dir
            .map(|r| r.expect("failed to read a directory entry"))
            .map(std::convert::TryInto::try_into)
            .filter_map(std::result::Result::ok)
            .collect::<Images>();
        assert!(
            images.len() > 0,
            "the images directory must contain at least 1 image."
        );
        Arguments {
            image_dir: images,
            names,
            names_dir,
        }
    }
}

/// # Errors
/// This errors for many reasons:
/// - if the number of arguments is wrong
/// - if the first argument path does not exist
/// - if the second argument path does not exist
/// - if the first argument is not a directory
/// - if the second argument is not a file
/// - if the second arguments extension is not .names
pub fn wrangle_args(args: Args) -> Result<Arguments, ArgumentError> {
    let args = args.collect::<Vec<_>>();
    if let [_, dir_path, names_path, _optional @ ..] = args.as_slice() {
        let dir = Path::new(dir_path);
        let names = Path::new(names_path);
        if !dir.exists() {
            Err(ArgumentError::FileDoesNotExist(format!(
                "the directory {} does not exist",
                dir_path
            )))
        } else if !names.exists() {
            Err(ArgumentError::FileDoesNotExist(format!(
                "the file {} does not exist",
                names_path
            )))
        } else if !dir.is_dir() {
            Err(ArgumentError::InvalidFileType(format!(
                "{} is not a directory",
                dir_path
            )))
        } else if !names.is_file() {
            Err(ArgumentError::InvalidFileType(format!(
                "{} is not a file",
                names_path
            )))
        } else if names.extension() == Some(OsStr::new("names")) {
            let images_directory = match dir.read_dir() {
                Ok(dir) => dir,
                Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
            };
            let names_directory = names
                .parent()
                .expect("names file must have a parent directory");
            let names = match File::open(names) {
                Ok(f) => match BufReader::new(f)
                    .lines()
                    .collect::<Result<Vec<String>, _>>()
                {
                    Ok(lines) => lines,
                    Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
                },
                Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
            };
            Ok(Arguments::new(
                images_directory,
                names,
                PathBuf::from(names_directory),
            ))
        } else {
            Err(ArgumentError::InvalidFileType(format!(
                "{} is not a names file",
                names_path
            )))
        }
    } else {
        Err(ArgumentError::InvalidNumber(format!(
            "expected arguments of the format <images directory> <names file> [optional args]. found {}: [\"{}\"]",
            args.len(),
            args.join("\",\"")
        )))
    }
}

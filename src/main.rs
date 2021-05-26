#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![warn(clippy::pedantic, rust_2018_idioms)]

use std::env::Args;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{File, ReadDir};
use std::io::{BufRead, BufReader, Error};
use std::ops::Add;
use std::path::Path;
use std::{env, fmt};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]

fn main() {
    match wrangle_args(env::args()) {
        Ok((directory, names)) => {
            let app = egui_template::RsMark::yolo(directory, names);
            let native_options = eframe::NativeOptions::default();
            eframe::run_native(Box::new(app), native_options);
        }
        Err(explanation) => {
            println!("{}", explanation)
        }
    }
}

enum ArgumentError {
    InvalidNumber(String),
    FileDoesNotExist(String),
    InvalidFileType(String),
    ReadError(String),
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (err, explanation) = match self {
            ArgumentError::InvalidNumber(i) => ("InvalidNumber", i),
            ArgumentError::FileDoesNotExist(i) => ("FileDoesNotExist", i),
            ArgumentError::InvalidFileType(i) => ("InvalidFileType", i),
            ArgumentError::ReadError(i) => ("ReadError", i),
        };
        write!(f, "{}: {}", err, explanation)
    }
}

fn wrangle_args(args: Args) -> Result<(ReadDir, Vec<String>), ArgumentError> {
    let vec = args.collect::<Vec<_>>();
    if let [_, dir_path, names_path] = vec.as_slice() {
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
        } else if names.extension() != Some(OsStr::new("names")) {
            Err(ArgumentError::InvalidFileType(format!(
                "{} is not a names file",
                names_path
            )))
        } else {
            let images_directory = match dir.read_dir() {
                Ok(dir) => dir,
                Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
            };
            let names = match File::open(names) {
                Ok(f) => match BufReader::new(f).lines().collect::<Result<Vec<_>, _>>() {
                    Ok(lines) => {
                        if lines.iter().all(|line| !line.trim_end().contains(" ")) {
                            lines
                        } else {
                            return Err(ArgumentError::ReadError(format!(
                                "{:?} contained a name with a space",
                                lines
                            )));
                        }
                    }
                    Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
                },
                Err(err) => return Err(ArgumentError::ReadError(err.to_string())),
            };
            Ok((images_directory, names))
        }
    } else {
        Err(ArgumentError::InvalidNumber(format!(
            "expected 3 arguments. found {}: [\"{}\"]",
            vec.len(),
            vec.join("\",\"")
        )))
    }
}

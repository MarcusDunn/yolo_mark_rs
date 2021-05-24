use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::num::{ParseFloatError, ParseIntError};
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;

use crate::app::label::yolo::bounding_box::BoundingBox;
use crate::app::label::Labels;

struct YoloLabels {
    boxes: Vec<BoundingBox>,
}

impl YoloLabels {
    fn new(boxes: Vec<BoundingBox>) -> Self {
        Self { boxes }
    }
}

impl TryFrom<File> for YoloLabels {
    type Error = YoloParseError;

    fn try_from(value: File) -> Result<Self, Self::Error> {
        BufReader::new(value)
            .lines()
            .map(|l| {
                l.map_err(YoloParseError::IoError)
                    .and_then(|str| BoundingBox::try_from(str))
            })
            .collect::<Result<Vec<BoundingBox>, YoloParseError>>()
            .map(|vec| YoloLabels::new(vec))
    }
}

impl Labels for YoloLabels {
    type Label = BoundingBox;

    fn add(&mut self, label: Self::Label) {
        todo!()
    }

    fn from_file(path: &Path) -> Option<Self> {
        todo!()
    }

    fn to_file(&self, path: &Path) {
        match File::create(path) {
            Ok(mut file) => {
                file.write_all(
                    self.boxes
                        .iter()
                        .map(
                            |BoundingBox {
                                 label,
                                 x,
                                 y,
                                 width,
                                 height,
                             }| {
                                format!("{} {} {} {} {}", label, x, y, width, height)
                            },
                        )
                        .collect::<Vec<_>>()
                        .join("\n")
                        .as_bytes(),
                )
                .expect("file to write");
            }
            Err(_) => {}
        }
    }
}

mod bounding_box;

pub enum YoloParseError {
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
    IoError(std::io::Error),
    MalformedLine(String),
}

impl From<ParseIntError> for YoloParseError {
    fn from(it: ParseIntError) -> Self {
        YoloParseError::ParseIntError(it)
    }
}

impl From<ParseFloatError> for YoloParseError {
    fn from(it: ParseFloatError) -> Self {
        YoloParseError::ParseFloatError(it)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels_to_file() {
        let labels = YoloLabels {
            boxes: vec![
                BoundingBox::new(1, 0.5, 0.5, 0.2, 0.3).unwrap(),
                BoundingBox::new(2, 0.5, 0.7, 0.2, 0.3).unwrap(),
            ],
        };
        let path = Path::new("data/test.txt");
        labels.to_file(path);
        let lines = BufReader::new(File::open(path).unwrap())
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();
        std::fs::remove_file(path);
        assert_eq!(lines, vec!["1 0.5 0.5 0.2 0.3", "2 0.5 0.7 0.2 0.3"]);
    }
}

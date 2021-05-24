use std::convert::TryFrom;
use std::fs::File;
use std::path::Path;

mod yolo;

trait Labels: TryFrom<File> {
    type Label;

    fn add(&mut self, label: Self::Label);
    fn from_file(path: &Path) -> Option<Self>;
    fn to_file(&self, path: &Path);
}

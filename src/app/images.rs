use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use crate::app::image_file::ImageFile;

/// A collection of *sorted* image files
pub struct Images(Vec<Image>);

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Image {
    pub img: ImageFile,
    pub marked: bool,
}

impl IndexMut<usize> for Images {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Images {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&ImageFile> {
        self.0.get(index).map(|it| &it.img)
    }

    pub(crate) fn as_slice(&self) -> &[Image] {
        self.0.as_slice()
    }
}

impl FromIterator<ImageFile> for Images {
    fn from_iter<T: IntoIterator<Item = ImageFile>>(iter: T) -> Self {
        let mut vec = iter
            .into_iter()
            .map(|img| Image { img, marked: false })
            .collect::<Vec<_>>();
        vec.sort();
        Images(vec)
    }
}

impl Index<usize> for Images {
    type Output = Image;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

use crate::app::image_file::ImageFile;
use std::iter::FromIterator;
use std::ops::Index;

/// A collection of *sorted* image files
pub struct Images(Vec<ImageFile>);

impl Images {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&ImageFile> {
        self.0.get(index)
    }

    pub(crate) fn as_slice(&self) -> &[ImageFile] {
        self.0.as_slice()
    }
}

impl FromIterator<ImageFile> for Images {
    fn from_iter<T: IntoIterator<Item = ImageFile>>(iter: T) -> Self {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        vec.sort();
        Images(vec)
    }
}

impl Index<usize> for Images {
    type Output = ImageFile;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

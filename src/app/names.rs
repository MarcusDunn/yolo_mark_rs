use std::iter::FromIterator;
use std::ops::Deref;

pub struct Names(Vec<String>);

impl Names {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromIterator<String> for Names {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Names {
        Names(iter.into_iter().collect())
    }
}

impl Deref for Names {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

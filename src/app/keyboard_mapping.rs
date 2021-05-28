use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::ops::Index;

use eframe::egui::Key;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Action {
    NextImage,
    PrevImage,
    NextName,
    PrevName,
}

pub struct KeyboardMapping(BTreeMap<Action, Key>);

impl KeyboardMapping {
    fn default_mappings() -> Vec<(Action, Key)> {
        vec![
            (Action::NextImage, Key::ArrowRight),
            (Action::PrevImage, Key::ArrowLeft),
            (Action::NextName, Key::ArrowUp),
            (Action::PrevName, Key::ArrowDown),
        ]
    }
}

impl FromIterator<(Action, Key)> for KeyboardMapping {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Action, Key)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

impl Index<Action> for KeyboardMapping {
    type Output = Key;

    fn index(&self, index: Action) -> &Self::Output {
        self.0
            .get(&index)
            .unwrap_or_else(|| panic!("key not bound for {:?}", index))
    }
}

impl Default for KeyboardMapping {
    fn default() -> Self {
        Self(BTreeMap::from_iter(Self::default_mappings()))
    }
}

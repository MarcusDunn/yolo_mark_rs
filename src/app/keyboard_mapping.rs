use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::ops::Index;

use eframe::egui::{CtxRef, Key};
use eframe::epi::egui::PointerButton;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Action {
    NextImage,
    PrevImage,
    NextName,
    PrevName,
    DragBox,
    RemoveBox,
}

pub enum EventTrigger {
    Key(Key),
    PointerButton(PointerButton),
}

impl From<Key> for EventTrigger {
    fn from(key: Key) -> Self {
        EventTrigger::Key(key)
    }
}

impl From<PointerButton> for EventTrigger {
    fn from(pointer_button: PointerButton) -> Self {
        EventTrigger::PointerButton(pointer_button)
    }
}

pub struct KeyboardMapping(BTreeMap<Action, EventTrigger>);

impl KeyboardMapping {
    pub(crate) fn is_triggered(&self, p0: Action, ctx: &CtxRef) -> bool {
        match self.0[&p0] {
            EventTrigger::Key(k) => ctx.input().key_pressed(k),
            EventTrigger::PointerButton(pb) => ctx.input().pointer.button_down(pb),
        }
    }
}

impl KeyboardMapping {
    fn default_mappings() -> Vec<(Action, EventTrigger)> {
        vec![
            (Action::NextImage, Key::ArrowRight.into()),
            (Action::PrevImage, Key::ArrowLeft.into()),
            (Action::NextName, Key::ArrowUp.into()),
            (Action::PrevName, Key::ArrowDown.into()),
            (Action::DragBox, PointerButton::Middle.into()),
            (Action::RemoveBox, Key::R.into()),
        ]
    }
}

impl FromIterator<(Action, EventTrigger)> for KeyboardMapping {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Action, EventTrigger)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

impl Index<Action> for KeyboardMapping {
    type Output = EventTrigger;

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

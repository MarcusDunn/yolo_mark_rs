use std::collections::BTreeMap;
use std::convert::TryInto;
use std::iter::FromIterator;
use std::ops::{Deref, Index};

use eframe::egui::{CtxRef, Key};
use eframe::epi::egui::PointerButton;

use crate::app::keyboard_mapping::zero_to_nine::ZeroToNine;

pub mod zero_to_nine {
    use std::convert::TryFrom;

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ZeroToNine(pub u8);

    impl ZeroToNine {
        pub(crate) fn iter() -> Box<dyn Iterator<Item = ZeroToNine>> {
            Box::new((0..=9).map(|i| ZeroToNine::new(i).unwrap()))
        }
    }

    impl ZeroToNine {
        pub(crate) fn new(n: u8) -> Option<ZeroToNine> {
            if (0..=9).contains(&n) {
                Some(ZeroToNine(n))
            } else {
                None
            }
        }
    }

    impl TryFrom<u8> for ZeroToNine {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match Self::new(value) {
                None => Err(()),
                Some(v) => Ok(v),
            }
        }
    }
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Action {
    NextImage,
    PrevImage,
    NextName,
    PrevName,
    RemoveBox,
    NameNumber(ZeroToNine),
    Clear,
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

impl Deref for KeyboardMapping {
    type Target = BTreeMap<Action, EventTrigger>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl KeyboardMapping {
    fn default_mappings() -> Vec<(Action, EventTrigger)> {
        vec![
            (Action::NextImage, Key::D.into()),
            (Action::PrevImage, Key::A.into()),
            (Action::NextName, Key::S.into()),
            (Action::PrevName, Key::W.into()),
            (Action::RemoveBox, Key::R.into()),
            (Action::NameNumber(0.try_into().unwrap()), Key::Num0.into()),
            (Action::NameNumber(1.try_into().unwrap()), Key::Num1.into()),
            (Action::NameNumber(2.try_into().unwrap()), Key::Num2.into()),
            (Action::NameNumber(3.try_into().unwrap()), Key::Num3.into()),
            (Action::NameNumber(4.try_into().unwrap()), Key::Num4.into()),
            (Action::NameNumber(5.try_into().unwrap()), Key::Num5.into()),
            (Action::NameNumber(6.try_into().unwrap()), Key::Num6.into()),
            (Action::NameNumber(7.try_into().unwrap()), Key::Num7.into()),
            (Action::NameNumber(8.try_into().unwrap()), Key::Num8.into()),
            (Action::NameNumber(9.try_into().unwrap()), Key::Num9.into()),
            (Action::Clear, Key::C.into()),
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
        &self.0[&index]
    }
}

impl Default for KeyboardMapping {
    fn default() -> Self {
        Self(BTreeMap::from_iter(Self::default_mappings()))
    }
}

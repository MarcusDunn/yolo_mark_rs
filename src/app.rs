use std::convert::TryInto;
use std::fs::ReadDir;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::time::{Duration, SystemTime};

use eframe::egui::{CtxRef, TextureId, Vec2};
use eframe::epi::Frame;
use eframe::{egui, epi};

use crate::app::image_cache::{ImageCache, ImageLookup};
use crate::app::image_file::ImageFile;
use crate::app::keyboard_mapping::KeyboardMapping;

pub mod keyboard_mapping {
    use std::collections::BTreeMap;
    use std::iter::FromIterator;
    use std::ops::Index;

    use eframe::egui::Key;

    use crate::app::Action;

    pub struct KeyboardMapping(BTreeMap<Action, Key>);

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
            Self(BTreeMap::from_iter(vec![
                (Action::NextImage, Key::ArrowRight),
                (Action::PrevImage, Key::ArrowLeft),
            ]))
        }
    }
}

pub struct RsMark {
    key_map: KeyboardMapping,
    current_index: AtomicUsize,
    images: Vec<ImageFile>,
    names: Vec<String>,
    image_cache: ImageCache,
    current_image: Option<(TextureId, Vec2)>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Action {
    NextImage,
    PrevImage,
}

mod image_cache;
mod image_file;

impl RsMark {
    pub fn yolo(directory: ReadDir, names: Vec<String>, key_map: KeyboardMapping) -> RsMark {
        let images = directory
            .map(|r| r.expect("failed to read a directory entry"))
            .map(|r| r.try_into())
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        println!("found {} images!", images.len());
        RsMark {
            key_map,
            current_index: AtomicUsize::new(0),
            images,
            names,
            image_cache: ImageCache::new(Vec2::new(500.0, 500.0)),
            current_image: None,
        }
    }

    pub fn handle_next(&mut self) {
        let index = self.current_index.fetch_add(1, Ordering::AcqRel);
        self.handle_new_image(index)
    }

    pub fn handle_prev(&mut self) {
        let index = self.current_index.fetch_sub(1, Ordering::AcqRel);
        self.handle_new_image(index)
    }

    pub fn handle_new_image(&mut self, index: usize) {
        self.current_image = None;
        println!("on image {:?}, {:?}", index, self.images.get(index))
    }
}

impl epi::App for RsMark {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        self.image_cache.update();
        egui::TopPanel::top("top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            })
        });
        if ctx.input().key_pressed(self.key_map[Action::NextImage]) {
            self.handle_next()
        }
        if ctx.input().key_pressed(self.key_map[Action::PrevImage]) {
            self.handle_prev()
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image_cache.set_size(ui.available_size()) {
                self.handle_new_image(self.current_index.load(Ordering::Relaxed))
            }
            if let Some((texture_id, size)) = self.current_image {
                ui.image(texture_id, size);
            } else {
                let get_result = self.image_cache.get(
                    ImageLookup {
                        index: self.current_index.load(Ordering::Relaxed),
                    },
                    self.images.as_slice(),
                );
                match get_result {
                    None => {
                        ui.label("damn I'm shit at coding");
                    }
                    Some(img) => {
                        self.current_image = Some((
                            frame
                                .tex_allocator()
                                .alloc_srgba_premultiplied(img.size_usize(), img.data.as_slice()),
                            img.size_vec2(),
                        ));
                    }
                }
            }
        });
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        self.handle_new_image(0);
    }

    fn name(&self) -> &str {
        "rs mark"
    }
}

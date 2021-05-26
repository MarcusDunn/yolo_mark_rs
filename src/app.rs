use std::convert::TryInto;
use std::fs::ReadDir;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::time::{Duration, SystemTime};

use eframe::egui::{CtxRef, Key, TextureId, Vec2};
use eframe::epi::Frame;
use eframe::{egui, epi};

use crate::app::image_cache::{ImageCache, ImageLookup};
use crate::app::image_file::ImageFile;

pub struct RsMark {
    current_index: AtomicUsize,
    images: Vec<ImageFile>,
    names: Vec<String>,
    image_cache: ImageCache,
    current_image: Option<(TextureId, Vec2)>,
}

mod image_cache;
mod image_file;

impl RsMark {
    pub fn yolo(directory: ReadDir, names: Vec<String>) -> RsMark {
        let images = directory
            .map(|r| r.expect("failed to read a directory entry"))
            .map(|r| r.try_into())
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        println!("found {} images!", images.len());
        RsMark {
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
                if ui.button("Next").clicked() || ctx.input().key_pressed(Key::ArrowRight) {
                    self.handle_next()
                }
                if ui.button("Prev").clicked() || ctx.input().key_pressed(Key::ArrowLeft) {
                    self.handle_prev()
                }
            })
        });
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

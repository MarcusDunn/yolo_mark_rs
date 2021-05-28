use std::convert::TryInto;
use std::fs::ReadDir;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::time::{Duration, SystemTime};

use eframe::egui::{CtxRef, Pos2, Rect, TextureId, Ui, Vec2};
use eframe::epi::Frame;
use eframe::{egui, epi};

use crate::app::bbox::BBox;
use crate::app::image_cache::{ImageCache, ImageLookup};
use crate::app::image_file::ImageFile;
use crate::app::keyboard_mapping::{Action, KeyboardMapping};

mod image_cache;
mod image_file;
pub mod keyboard_mapping;

pub struct RsMark {
    key_map: KeyboardMapping,
    current_index: AtomicUsize,
    images: Vec<ImageFile>,
    names: Vec<String>,
    selected_name: usize,
    image_cache: ImageCache,
    current_image: Option<(TextureId, Vec2)>,
    current_boxes: Vec<BBox>,
}

mod bbox;

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
            selected_name: 0,
            image_cache: ImageCache::new(Vec2::new(500.0, 500.0)),
            current_image: None,
            current_boxes: vec![
                BBox::new(1, 0.9, 0.8, 0.5, 0.5).expect("invalid box"),
                BBox::new(1, 0.5, 0.5, 0.4, 0.6).expect("invalid box"),
            ],
        }
    }

    pub fn handle_index_change(&mut self, incr: isize) {
        if incr.is_negative() {
            self.current_index
                .fetch_sub(incr.abs() as usize, Ordering::AcqRel);
        } else {
            self.current_index
                .fetch_add(incr.abs() as usize, Ordering::AcqRel);
        }
        self.current_image = None;
    }
}

impl epi::App for RsMark {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        self.image_cache.update();
        self.handle_key_presses(ctx);
        self.menu_bar(ctx, frame);
        self.display_images(ctx, frame);
    }

    fn name(&self) -> &str {
        "rs mark"
    }
}

impl RsMark {
    fn handle_key_presses(&mut self, ctx: &CtxRef) {
        if ctx.input().key_pressed(self.key_map[Action::NextImage]) {
            self.handle_index_change(1)
        }
        if ctx.input().key_pressed(self.key_map[Action::PrevImage]) {
            self.handle_index_change(-1)
        }
        if ctx.input().key_pressed(self.key_map[Action::NextName]) {
            self.selected_name += 1;
            if self.selected_name >= self.names.len() {
                self.selected_name = 0;
            }
        }
        if ctx.input().key_pressed(self.key_map[Action::PrevName]) {
            self.selected_name = if self.selected_name == 0 {
                self.names.len() - 1
            } else {
                self.selected_name - 1
            }
        }
    }
}

impl RsMark {
    fn menu_bar(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::TopPanel::top("top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                for i in 0..self.names.len() {
                    if ui
                        .selectable_label(self.selected_name == i, &self.names[i])
                        .clicked()
                    {
                        self.selected_name = i
                    }
                }
            })
        });
    }
}

impl RsMark {
    fn display_images(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image_cache.set_size(ui.available_size()) {
                self.current_image = None
            }
            if let Some((texture_id, size)) = self.current_image {
                ui.image(texture_id, size);
                self.paint_boxes(&ui, size)
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

    fn paint_boxes(&mut self, ui: &&mut Ui, size: Vec2) {
        let top_left = ui.clip_rect().min;
        let rect = Rect {
            min: top_left,
            max: Pos2 {
                x: top_left.x + size.x,
                y: top_left.y + size.y,
            },
        };
        let painter = &mut ui.painter_at(rect);

        let mut covered_box: Option<&BBox> = None;
        for bbox in &self.current_boxes {
            let rect = bbox.draw(painter, 100);
            bbox.draw_text(painter, &self.names, rect, 100);
            if ui.rect_contains_pointer(rect) {
                if let Some(selected) = covered_box {
                    if selected.width > bbox.width && selected.height > bbox.height {
                        covered_box = Some(bbox);
                    }
                } else {
                    covered_box = Some(bbox)
                }
            }
        }
        if let Some(bbox) = covered_box {
            let rect = bbox.draw(painter, 0);
            bbox.draw_text(painter, &self.names, rect, 0);
        }
    }
}

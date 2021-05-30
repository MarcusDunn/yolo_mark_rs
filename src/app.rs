use std::convert::TryFrom;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::time::{Duration, SystemTime};

use eframe::egui::{
    CtxRef, ImageButton, InnerResponse, Pos2, Rect, Sense, TextEdit, TextureId, Ui, Vec2,
};
use eframe::epi::Frame;
use eframe::{egui, epi};

use crate::app::arguments::Arguments;
use crate::app::bbox::BBox;
use crate::app::image_cache::{ImageCache, ImageLookup};
use crate::app::images::Images;
use crate::app::keyboard_mapping::{Action, KeyboardMapping};
use crate::app::names::Names;

mod image_cache;
mod image_file;
pub mod keyboard_mapping;

pub struct RsMark {
    // index of box in current_boxes
    selected_box: Option<usize>,
    current_image_input_text: String,
    key_map: KeyboardMapping,
    current_index: AtomicUsize,
    images: Images,
    names: Names,
    selected_name: usize,
    image_cache: ImageCache,
    current_image: Option<(TextureId, Vec2)>,
    current_boxes: Vec<BBox>,
    drag_start: Option<Pos2>,
    drag_diff: Option<Pos2>,
}

mod names;

mod images;

impl RsMark {
    pub(crate) fn display_info(&mut self, ctx: &CtxRef) -> InnerResponse<()> {
        egui::TopPanel::top("top info panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let button_resp = ui.button("Jump to image:");
                let resp = ui.add(
                    TextEdit::singleline(&mut self.current_image_input_text).desired_width(10.0),
                );
                if button_resp.clicked() || resp.lost_focus() {
                    match self.current_image_input_text.parse::<usize>() {
                        Ok(index) => {
                            let curr = self.current_index.load(Ordering::SeqCst);
                            match (isize::try_from(index), isize::try_from(curr)) {
                                (Ok(index), Ok(curr)) => self.handle_index_change(index - curr),
                                _ => {
                                    self.current_image_input_text = String::from("index too high");
                                }
                            };
                        }
                        Err(err) => self.current_image_input_text = err.to_string(),
                    };
                }
            });
        })
    }
}

pub mod arguments;
mod bbox;

impl RsMark {
    pub fn yolo(Arguments { image_dir, names }: Arguments, key_map: KeyboardMapping) -> RsMark {
        println!("found {} images!", image_dir.len());
        RsMark {
            selected_box: None,
            current_image_input_text: 0.to_string(),
            key_map,
            current_index: AtomicUsize::new(0),
            images: image_dir,
            names,
            selected_name: 0,
            image_cache: ImageCache::new(Vec2::new(500.0, 500.0)),
            current_image: None,
            current_boxes: Vec::new(),
            drag_start: None,
            drag_diff: None,
        }
    }

    pub fn handle_index_change(&mut self, incr: isize) {
        let index = if incr.is_negative() {
            self.current_index
                .fetch_sub(incr.abs() as usize, Ordering::SeqCst)
        } else {
            self.current_index
                .fetch_add(incr.abs() as usize, Ordering::SeqCst)
        };
        self.images[index]
            .save_labels(&self.current_boxes)
            .unwrap_or_else(|err| panic!("error occurred while writing label {}", err));
        self.current_boxes = self
            .images
            .get((index as isize + incr) as usize)
            .unwrap_or_else(|| {
                self.current_index.store(index, Ordering::SeqCst);
                &self.images[index]
            })
            .load_labels();
        let index = self.current_index.load(Ordering::SeqCst);
        self.current_image_input_text = index.to_string();
        self.current_image = None;
    }
}

impl epi::App for RsMark {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        self.image_cache.update();
        self.handle_key_presses(ctx);
        self.display_info(ctx);
        self.display_names(ctx);
        self.display_images(ctx, frame);
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        self.image_cache.update();
        self.current_boxes = self.images[self.current_index.load(Ordering::SeqCst)].load_labels();
    }

    fn on_exit(&mut self) {
        self.images[self.current_index.load(Ordering::SeqCst)]
            .save_labels(&self.current_boxes)
            .unwrap_or_else(|err| {
                panic!(
                    "FAILED TO SAVE FINAL ANNOTATIONS ON EXIT {:#?} \n\n DUE TO {}",
                    self.current_boxes, err
                )
            });
    }

    fn name(&self) -> &str {
        "rs mark"
    }
}

impl RsMark {
    fn handle_key_presses(&mut self, ctx: &CtxRef) {
        if self.key_map.is_triggered(Action::NextImage, ctx) {
            self.handle_index_change(1)
        }
        if self.key_map.is_triggered(Action::PrevImage, ctx) {
            self.handle_index_change(-1)
        }
        if self.key_map.is_triggered(Action::NextName, ctx) {
            self.selected_name += 1;
            if self.selected_name >= self.names.len() {
                self.selected_name = 0;
            }
        }
        if self.key_map.is_triggered(Action::PrevName, ctx) {
            self.selected_name = if self.selected_name == 0 {
                self.names.len() - 1
            } else {
                self.selected_name - 1
            }
        }
        if let Some(box_inx) = self.selected_box {
            if self.key_map.is_triggered(Action::RemoveBox, ctx) {
                self.current_boxes.remove(box_inx);
            }
        }
    }
}

impl RsMark {
    fn display_images(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) -> InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image_cache.set_size(ui.available_size()) {
                self.handle_index_change(0)
            }
            if let Some((texture_id, size)) = self.current_image {
                let img = ImageButton::new(texture_id, size)
                    .sense(Sense::click_and_drag())
                    .frame(false);
                let img_resp = ui.add(img);
                let rect = Rect {
                    min: img_resp.rect.min,
                    max: Pos2 {
                        x: img_resp.rect.min.x + size.x,
                        y: img_resp.rect.min.y + size.y,
                    },
                };
                if img_resp.drag_started() {
                    self.drag_start = img_resp.interact_pointer_pos();
                    self.drag_diff = Some(Pos2::ZERO)
                }
                if let Some(curr_drag_diff) = self.drag_diff {
                    self.drag_diff = Some(curr_drag_diff + img_resp.drag_delta())
                }
                if img_resp.drag_released() {
                    if let (Some(drag_srt), Some(drag_diff)) = (self.drag_start, self.drag_diff) {
                        match BBox::from_two_points_and_rect(
                            self.selected_name,
                            rect,
                            drag_srt,
                            drag_diff,
                        ) {
                            Ok(bbox) => self.current_boxes.push(bbox),
                            Err(err) => println!("error creating box {}", err),
                        }
                    }
                    self.drag_diff = None;
                    self.drag_start = None;
                }
                self.paint_boxes(&ui, rect)
            } else {
                let get_result = self.image_cache.get(
                    ImageLookup {
                        index: self.current_index.load(Ordering::SeqCst),
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
        })
    }

    fn paint_boxes(&mut self, ui: &&mut Ui, rect: Rect) {
        let painter = &mut ui.painter_at(rect);
        self.selected_box = None;
        if let (Some(drag_start), Some(drag_diff)) = (self.drag_start, self.drag_diff) {
            match BBox::from_two_points_and_rect(self.selected_name, rect, drag_start, drag_diff) {
                Ok(bbox) => {
                    bbox.draw(painter, 0, true);
                }
                Err(err) => {
                    println!("WARNING: error when creating box from drag {}", err);
                    println!("ignoring for now . . .")
                }
            }
        }
        for (i, bbox) in self.current_boxes.iter().enumerate() {
            let rect = bbox.draw(painter, 100, false);
            bbox.draw_text(painter, &self.names, rect, 100, false);
            if ui.rect_contains_pointer(rect) {
                if let Some(selected) = self.selected_box {
                    if self.current_boxes[selected].width > bbox.width
                        && self.current_boxes[selected].height > bbox.height
                    {
                        self.selected_box = Some(i);
                    }
                } else {
                    self.selected_box = Some(i)
                }
            }
        }
        if let Some(bbox) = self.selected_box {
            let rect = self.current_boxes[bbox].draw(painter, 255, true);
            self.current_boxes[bbox].draw_text(painter, &self.names, rect, 255, true);
        }
    }
}

impl RsMark {
    fn display_names(&mut self, ctx: &CtxRef) {
        egui::SidePanel::left("side panel", 200.0).show(ctx, |ui| {
            egui::ScrollArea::auto_sized()
                .always_show_scroll(false)
                .show(ui, |ui| {
                    for i in 0..self.names.len() {
                        let names_resp =
                            ui.selectable_label(self.selected_name == i, &self.names[i]);
                        if names_resp.clicked() {
                            self.selected_name = i
                        }
                    }
                });
        });
    }
}

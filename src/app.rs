use std::convert::TryFrom;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
pub use std::time::{Duration, SystemTime};

use eframe::egui::{
    Align, Align2, CentralPanel, Color32, CtxRef, Image, InnerResponse, Painter, Pos2, Rect, Sense,
    Stroke, TextEdit, TextStyle, TextureId, Ui, Vec2,
};
use eframe::epi::Frame;
use eframe::{egui, epi};

use crate::app::arguments::Arguments;
use crate::app::bbox::{BBox, BBoxError};
use crate::app::drag_status::DragStatus;
use crate::app::image_cache::{ImageCache, ImageLookup};
use crate::app::images::Images;
use crate::app::keyboard_mapping::zero_to_nine::ZeroToNine;
use crate::app::keyboard_mapping::{Action, KeyboardMapping};
use crate::app::settings::Settings;

mod drag_status;
mod image_cache;
mod image_file;
mod images;
pub mod keyboard_mapping;
mod settings;

pub struct RsMark {
    // index of box in current_boxes
    page: Page,
    settings: Settings,
    selected_box: Option<usize>,
    current_image_input_text: String,
    key_map: KeyboardMapping,
    current_index: AtomicUsize,
    images: Images,
    names: Vec<String>,
    selected_name: usize,
    image_cache: ImageCache,
    current_image: Option<(TextureId, Vec2)>,
    current_boxes: Vec<BBox>,
    drag: DragStatus,
    shortcut_buffer: Vec<(ZeroToNine, Instant)>,
}

impl RsMark {
    pub(crate) fn display_edit_settings(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::TopPanel::top("top info panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File ", |ui| {
                    if ui.button("Label").clicked() {
                        self.page = Page::Label;
                    };
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            })
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.label("key_combo_trigger_ms");
            let mut key_combo_trigger_ms = self.settings.key_combo_trigger_ms.to_string();
            if ui.text_edit_singleline(&mut key_combo_trigger_ms).changed() {
                if let Ok(new) = key_combo_trigger_ms.parse() {
                    self.settings.key_combo_trigger_ms = new;
                }
            }
        });
    }
}

enum Page {
    Label,
    Settings,
}

impl RsMark {
    pub(crate) fn display_info(
        &mut self,
        ctx: &CtxRef,
        frame: &mut Frame<'_>,
    ) -> InnerResponse<()> {
        egui::TopPanel::top("top info panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File ", |ui| {
                    if ui.button("Settings").clicked() {
                        self.page = Page::Settings;
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                if ui.button("Next").clicked() {
                    self.handle_index_change(1);
                } else if ui.button("Prev").clicked() {
                    self.handle_index_change(-1);
                }
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
    #[must_use]
    pub fn yolo(Arguments { image_dir, names }: Arguments, key_map: KeyboardMapping) -> RsMark {
        println!("found {} images!", image_dir.len());
        RsMark {
            page: Page::Label,
            settings: Settings::from_file().unwrap_or_default(),
            selected_box: None,
            current_image_input_text: 0.to_string(),
            key_map,
            current_index: AtomicUsize::new(0),
            images: image_dir,
            names,
            selected_name: 0,
            image_cache: ImageCache::new(Vec2::new(500_f32, 500_f32)),
            current_image: None,
            current_boxes: Vec::new(),
            drag: DragStatus::empty(),
            shortcut_buffer: Vec::new(),
        }
    }

    /// # Panics
    /// this will panic if for whatever reason, the recently drawn labels fail to save.
    /// This is intended to prevent the user from labeling images for hours and nothing saving.
    pub fn handle_index_change(&mut self, incr: isize) {
        let mut reverted_index = false;
        let prev_index = if incr.is_negative() {
            self.current_index
                .fetch_sub(incr.abs() as usize, Ordering::SeqCst)
        } else {
            self.current_index
                .fetch_add(incr.abs() as usize, Ordering::SeqCst)
        };
        let new_index = self.current_index.load(Ordering::SeqCst);
        self.images[prev_index]
            .save_labels(&self.current_boxes)
            .unwrap_or_else(|err| panic!("error occurred while writing label {}", err));
        self.current_boxes = self
            .images
            .get(new_index)
            .unwrap_or_else(|| {
                // restores old index value that we know is valid.
                self.current_index.store(prev_index, Ordering::SeqCst);
                reverted_index = true;
                &self.images[prev_index]
            })
            .load_labels();
        self.current_image_input_text = {
            if reverted_index {
                prev_index
            } else {
                new_index
            }
        }
        .to_string();
        self.current_image = None;
    }
}

impl epi::App for RsMark {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        match &self.page {
            Page::Label => {
                self.image_cache.update();
                self.handle_key_presses(ctx);
                self.display_info(ctx, frame);
                self.display_names(ctx);
                self.display_images(ctx, frame);
            }
            Page::Settings => self.display_edit_settings(ctx, frame),
        }
    }

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        self.image_cache.update();
        self.current_boxes = self.images[self.current_index.load(Ordering::SeqCst)].load_labels();
    }

    fn on_exit(&mut self) {
        self.images[self.current_index.load(Ordering::SeqCst)]
            .save_labels(&self.current_boxes)
            .unwrap_or_else(|err| {
                println!(
                    "FAILED TO SAVE FINAL ANNOTATIONS ON EXIT {:#?} \n\n DUE TO {}",
                    self.current_boxes, err
                );
            });
        match File::with_options()
            .create(true)
            .write(true)
            .open("settings.json")
        {
            Ok(f) => {
                let clear_result = f.set_len(0);
                let mut lw = LineWriter::new(f);
                lw.write_all(
                    serde_json::to_string(&self.settings)
                        .expect("failed to parse settings")
                        .as_bytes(),
                )
                .expect("FAILED TO WRITE SETTINGS ");
                clear_result.expect("failed to clear the file before writing settings, this has possibly left the settings file in an invalid state");
            }
            Err(err) => {
                println!(
                    "FAILED TO SAVE SETTINGS ON EXIT {:#?} \n\n due to {}",
                    self.settings, err
                );
            }
        }
    }

    fn name(&self) -> &str {
        "RS Mark"
    }
}

impl RsMark {
    fn handle_key_presses(&mut self, ctx: &CtxRef) {
        if self.key_map.is_triggered(Action::Clear, ctx) {
            self.current_boxes.clear();
        }
        if self.key_map.is_triggered(Action::NextImage, ctx) {
            self.handle_index_change(1);
        }
        if self.key_map.is_triggered(Action::PrevImage, ctx) {
            self.handle_index_change(-1);
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
        if let Some((_, t)) = self.shortcut_buffer.last() {
            if Instant::now().duration_since(*t).as_millis() > self.settings.key_combo_trigger_ms {
                let shortcut = self
                    .shortcut_buffer
                    .iter()
                    .fold(String::new(), |acc, (ZeroToNine(n), _)| {
                        format!("{}{}", acc, n)
                    })
                    .parse::<usize>()
                    .unwrap();
                if self.names.len() > shortcut {
                    self.selected_name = shortcut;
                }
                self.shortcut_buffer.clear();
            }
        }
        for i in ZeroToNine::iter() {
            if self.key_map.is_triggered(Action::NameNumber(i), ctx) {
                self.shortcut_buffer.push((i, Instant::now()));
            }
        }

        if ctx.input().scroll_delta.y < 0.0 {
            self.selected_name = if self.selected_name + 1 >= self.names.len() {
                0
            } else {
                self.selected_name + 1
            }
        } else if ctx.input().scroll_delta.y > 0.0 {
            self.selected_name = if self.selected_name == 0 {
                self.names.len()
            } else {
                self.selected_name - 1
            }
        }
    }
}

impl RsMark {
    fn display_images(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) -> InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image_cache.set_size(ui.available_size()) {
                self.handle_index_change(0);
            }
            if let Some((texture_id, size)) = self.current_image {
                let img = Image::new(texture_id, size).sense(Sense::click_and_drag());
                let img_resp = ui.add(img);
                let rect = Rect {
                    min: img_resp.rect.min,
                    max: Pos2 {
                        x: img_resp.rect.min.x + size.x,
                        y: img_resp.rect.min.y + size.y,
                    },
                };
                if img_resp.drag_started() {
                    self.drag.start(img_resp.interact_pointer_pos().unwrap());
                }
                if let Some(curr_drag_diff) = self.drag.drag_diff {
                    self.drag.drag_diff = Some(curr_drag_diff + img_resp.drag_delta());
                }
                if img_resp.drag_released() {
                    if let (Some(drag_srt), Some(drag_diff)) =
                        (self.drag.drag_start, self.drag.drag_diff)
                    {
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
                    self.drag.clear();
                }
                let painter = &mut ui.painter_at(rect);
                self.paint_boxes(&ui, painter);
                if let Some(pos) = ctx.input().pointer.hover_pos() {
                    if let Some(text) = self.names.get(self.selected_name) {
                        let rect = painter.text(
                            pos,
                            Align2::CENTER_BOTTOM,
                            text,
                            TextStyle::Heading,
                            eframe::egui::Color32::BLACK,
                        );
                        painter.rect(rect, 0.0, Color32::from_white_alpha(100), Stroke::default());
                        painter.text(
                            pos,
                            Align2::CENTER_BOTTOM,
                            text,
                            TextStyle::Heading,
                            eframe::egui::Color32::BLACK,
                        );
                    }
                }
            } else {
                let get_result = self.image_cache.get(
                    ImageLookup {
                        index: self.current_index.load(Ordering::SeqCst),
                    },
                    self.images.as_slice(),
                );
                // if the image dimensions are larger than
                match get_result {
                    None => {
                        ui.label("Loading . . .");
                        ui.label("try moving your mouse to force an update!");
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

    fn paint_boxes(&mut self, ui: &&mut Ui, painter: &mut Painter) {
        self.selected_box = None;
        if let (Some(drag_start), Some(drag_diff)) = (self.drag.drag_start, self.drag.drag_diff) {
            match BBox::from_two_points_and_rect(
                self.selected_name,
                painter.clip_rect(),
                drag_start,
                drag_diff,
            ) {
                Ok(bbox) => {
                    bbox.draw(painter, 0, true);
                }
                Err(BBoxError::InvalidField(_)) => { /*ignore invalid boxes when dragging due to logging noise*/
                }
                Err(err) => {
                    println!("WARNING: error when creating box from drag {}", err);
                    println!("ignoring for now . . .");
                }
            }
        }
        for (i, bbox) in self.current_boxes.iter().enumerate() {
            let rect = bbox.draw(painter, 100, false);
            bbox.draw_text(painter, &self.names, rect, 100, false);
            if ui.rect_contains_pointer(rect) {
                if let Some(selected) = self.selected_box {
                    if self.current_boxes[selected].is_larger(bbox) {
                        self.selected_box = Some(i);
                    }
                } else {
                    self.selected_box = Some(i);
                }
            }
        }
        if let Some(bbox) = self.selected_box {
            let rect = self.current_boxes[bbox].draw(painter, 255, true);
            self.current_boxes[bbox].draw_text(painter, &self.names, rect, 255, true);
        };
    }
}

impl RsMark {
    fn display_names(&mut self, ctx: &CtxRef) {
        egui::SidePanel::left("side panel", 200.0).show(ctx, |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                for i in 0..self.names.len() {
                    let checked = self.selected_name == i;
                    let names_resp =
                        ui.selectable_label(checked, &format!("{}: {}", i, self.names[i]));
                    if checked {
                        names_resp.scroll_to_me(Align::Center);
                    }
                    if names_resp.clicked() {
                        self.selected_name = i;
                    }
                }
            });
        });
    }
}

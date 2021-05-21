use std::fs::{DirEntry, ReadDir};
use std::path::Path;

use eframe::egui::{Pos2, Rect, TextureId, Vec2};
use eframe::epi::Frame;
use eframe::{egui, epi};
use egui::Color32;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Pixel};

use crate::app::widgets::{BBox, DrawableImage};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    names: Vec<String>,
    name_field: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    current_image: DirEntry,
    #[cfg_attr(feature = "persistence", serde(skip))]
    _images_directory: ReadDir,
    boxes: Vec<BBox>,
    drag_start: Option<Pos2>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut images_directory = Path::new("data/img")
            .read_dir()
            .expect("data/img to be a directory");
        let entry = images_directory.next().unwrap().unwrap();
        Self {
            // Example stuff:
            names: Vec::new(),
            name_field: String::from("new name!"),
            current_image: entry,
            _images_directory: images_directory,
            boxes: Vec::new(),
            drag_start: None,
        }
    }
}

impl epi::App for TemplateApp {
    fn update<'b>(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'b>) {
        let Self {
            names, name_field, ..
        } = self;

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Names");
            ui.text_edit_singleline(name_field);
            if ui.button("add").clicked() {
                names.push(name_field.clone());
                name_field.clear();
            }

            ui.horizontal(|ui| {
                for name in names {
                    ui.label(name.to_string());
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Self {
                current_image,
                boxes,
                ..
            } = self;
            let resized = image::open(current_image.path().as_path()).unwrap().resize(
                ui.available_size().x as u32,
                ui.available_size().y as u32,
                FilterType::Nearest,
            );
            let img_size = Vec2 {
                x: resized.width() as f32,
                y: resized.height() as f32,
            };

            let texture_id = to_texture_id(frame, resized);
            let response = ui.add(DrawableImage::new(texture_id, img_size, boxes));

            if response.drag_released() {
                if let Some(start) = self.drag_start {
                    println!("start {:?}", start);
                    let end = response.interact_pointer_pos();
                    let Vec2 { x, y } = end.map(|p| p - response.rect.min).unwrap();
                    let adj_end = Pos2 { x, y };
                    println!("adj_end: {:?}", adj_end);
                    let bbox = Rect::from_two_pos(start, adj_end);
                    let scaled = BBox::new(response.rect.size(), bbox).unwrap();
                    println!("scaled {:?}", scaled);
                    self.boxes.push(scaled);
                    self.drag_start = None
                }
            } else if response.drag_started() {
                let Rect { min, .. } = response.rect;
                self.drag_start = response
                    .interact_pointer_pos()
                    .map(|pos| pos - min)
                    .map(|Vec2 { x, y }| Pos2 { x, y });
            }
        });
    }

    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "egui template"
    }
}

fn to_texture_id(tx_alloc: &'_ mut Frame<'_>, resized: DynamicImage) -> TextureId {
    tx_alloc.tex_allocator().alloc_srgba_premultiplied(
        (resized.width() as usize, resized.height() as usize),
        resized
            .pixels()
            .map(|(.., rgba)| {
                let (r, g, b, a) = rgba.channels4();
                Color32::from_rgba_premultiplied(r, g, b, a)
            })
            .collect::<Vec<_>>()
            .as_slice(),
    )
}

mod widgets;

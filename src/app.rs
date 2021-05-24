use std::time::SystemTime;

use eframe::egui::{Color32, CtxRef, TextureId, Vec2};
use eframe::epi::Frame;
use eframe::{egui, epi};
use image::{DynamicImage, GenericImageView, ImageError, Pixel};

use crate::app::settings::yolo::Yolo;
use crate::app::settings::{FileSetting, Setting, Settings};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

pub struct RsMark {
    desired_page: Page,
    settings: Settings,
    current_image_index: usize,
    current_image: Option<TextureId>,
}

impl RsMark {
    pub fn yolo() -> RsMark {
        RsMark {
            desired_page: Page::Settings,
            settings: Settings::yolo_default(),
            current_image_index: 0,
            current_image: None,
        }
    }
}

#[derive(PartialEq)]
enum Page {
    Label,
    Settings,
}

mod settings;

impl epi::App for RsMark {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        egui::TopPanel::top("top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                    if ui.button("Settings").clicked() {
                        self.desired_page = Page::Settings
                    }
                });
                if ui.button("next").clicked() {
                    self.current_image = None;
                    self.current_image_index = self
                        .current_image_index
                        .checked_add(1)
                        .unwrap_or(usize::MAX)
                }
                if ui.button("prev").clicked() {
                    self.current_image = None;
                    self.current_image_index = self.current_image_index.checked_sub(1).unwrap_or(0)
                }
            })
        });

        if self.desired_page == Page::Label && self.settings.is_valid() {
            egui::CentralPanel::default().show(ctx, |ui| match &self.settings {
                Settings::Yolo(Yolo { img_dir, .. }) => {
                    if let Ok(file_result) = img_dir.read_file() {
                        if let Some(curr_image) = &self.current_image {
                            ui.image(*curr_image, ui.available_size());
                        } else {
                            if let Some(current_image) = file_result.get(self.current_image_index) {
                                let start = std::time::SystemTime::now();
                                println!("opening img");
                                let image =
                                    image::open(current_image.path()).expect("image to open");
                                println!(
                                    "done opening image {:?}",
                                    SystemTime::now().duration_since(start)
                                );
                                let start = std::time::SystemTime::now();
                                println!("mapping pixels");
                                self.current_image = Some(
                                    frame.tex_allocator().alloc_srgba_premultiplied(
                                        (image.width() as usize, image.height() as usize),
                                        image
                                            .pixels()
                                            .map(|(.., p)| {
                                                let (r, g, b, a) = p.channels4();
                                                Color32::from_rgba_premultiplied(r, g, b, a)
                                            })
                                            .collect::<Vec<_>>()
                                            .as_slice(),
                                    ),
                                );
                                println!(
                                    "done mapping pixels {:?}",
                                    SystemTime::now().duration_since(start)
                                );
                            } else {
                            }
                        }
                    } else {
                        self.desired_page = Page::Settings
                    }
                }
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                let settings = &mut self.settings;
                settings.as_page(ui);
                if settings.is_valid() && ui.button("Done").clicked() {
                    self.desired_page = Page::Label
                }
            });
        }
    }

    fn name(&self) -> &str {
        "rs mark"
    }
}

mod label;

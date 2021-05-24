use eframe::egui::CtxRef;
use eframe::epi::Frame;
use eframe::{egui, epi};

pub use settings::yolo;

use crate::app::settings::{Setting, Settings};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

pub struct RsMark {
    desired_page: Page,
    settings: Settings,
}

impl RsMark {
    pub fn yolo() -> RsMark {
        RsMark {
            desired_page: Page::Settings,
            settings: Settings::yolo_default(),
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
                })
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.desired_page == Page::Label && self.settings.is_valid() {
                // label
            } else {
                let settings = &mut self.settings;
                settings.as_page(ui);
                if settings.is_valid() {
                    if ui.button("Done").clicked() {
                        self.desired_page = Page::Label
                    }
                }
            }
        });
    }

    fn name(&self) -> &str {
        "rs mark"
    }
}

mod label;

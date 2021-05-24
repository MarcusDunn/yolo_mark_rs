use std::fmt;
use std::fs::{File, ReadDir};
use std::io::{BufRead, BufReader, Error};
use std::path::Path;
use std::rc::Rc;

use image::io::Reader;

use crate::yolo::{ImagesDirectorySetting, NamesFileSetting};
use eframe::egui::Ui;

pub mod yolo;

pub enum Settings {
    Yolo(yolo::ImagesDirectorySetting, yolo::NamesFileSetting),
}

impl Settings {
    pub fn yolo_default() -> Settings {
        Settings::Yolo(
            ImagesDirectorySetting::default(),
            NamesFileSetting::default(),
        )
    }

    pub fn as_page(&mut self, ui: &mut Ui) {
        match self {
            Settings::Yolo(images_dir, names_file) => {
                ui.heading(format!(
                    "{}: {}",
                    ImagesDirectorySetting::SETTING_NAME,
                    images_dir.display_validity()
                ));
                ui.text_edit_singleline(images_dir.backing_path_mut());

                ui.heading(format!(
                    "{}: {}",
                    NamesFileSetting::SETTING_NAME,
                    names_file.display_validity()
                ));
                ui.text_edit_singleline(names_file.backing_path_mut());
            }
        }
    }
}

impl Setting for Settings {
    const SETTING_NAME: &'static str = "Settings";

    fn is_valid(&self) -> bool {
        match self {
            Settings::Yolo(img_dir, names_file) => img_dir.is_valid() && names_file.is_valid(),
        }
    }

    fn display_validity(&self) -> String {
        match self {
            Settings::Yolo(img_dir, names_file) => format!(
                "
        Image Directory {}
        Names File {}
        ",
                img_dir.display_validity(),
                names_file.display_validity()
            ),
        }
    }
}

pub trait FileSetting: Setting {
    type FileResult;
    type Error;

    fn read_file(&self) -> Result<Self::FileResult, Self::Error>;
    fn backing_path_mut(&mut self) -> &mut String;
}

pub trait Setting {
    const SETTING_NAME: &'static str;
    fn is_valid(&self) -> bool;
    fn display_validity(&self) -> String;
}

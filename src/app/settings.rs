use eframe::egui::Ui;

use crate::app::settings::yolo::{ImagesDirectorySetting, NamesFileSetting, Yolo};

pub mod yolo;

pub enum Settings {
    Yolo(Yolo),
}

impl Settings {
    pub fn yolo_default() -> Settings {
        Settings::Yolo(Yolo::default())
    }

    pub fn as_page(&mut self, ui: &mut Ui) {
        match self {
            Settings::Yolo(Yolo {
                img_dir,
                names_file,
            }) => {
                ui.heading(format!(
                    "{}: {}",
                    ImagesDirectorySetting::SETTING_NAME,
                    img_dir.display_validity()
                ));
                ui.text_edit_singleline(img_dir.backing_path_mut());

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
            Settings::Yolo(yolo) => yolo.is_valid(),
        }
    }

    fn display_validity(&self) -> String {
        match self {
            Settings::Yolo(Yolo {
                img_dir,
                names_file,
            }) => format!(
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

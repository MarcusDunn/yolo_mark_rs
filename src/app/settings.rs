use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU32;

trait Partial<T>
where
    T: Default,
{
    fn fill(&self) -> T;
}

#[derive(serde::Serialize, Debug)]
pub struct Settings {
    pub key_combo_trigger_ms: u128,
    pub cross_hair_alpha: u8,
    pub bounding_box_alpha: u8,
    pub display_bounding_box_name: bool,
    pub scroll_thresh: f32,
    pub start_img_index: usize,
    pub display_cursor_name: bool,
    pub save_interval_seconds: NonZeroU32,
    pub dynamic_crosshair: bool,
}

#[derive(serde::Deserialize)]
struct PartialSettings {
    key_combo_trigger_ms: Option<u128>,
    cross_hair_alpha: Option<u8>,
    bounding_box_alpha: Option<u8>,
    display_bounding_box_name: Option<bool>,
    scroll_thresh: Option<f32>,
    start_img_index: Option<usize>,
    display_cursor_name: Option<bool>,
    save_interval_seconds: Option<NonZeroU32>,
    dynamic_crosshair: Option<bool>,
}

impl Partial<Settings> for PartialSettings {
    fn fill(&self) -> Settings {
        let default = Settings::default();
        Settings {
            key_combo_trigger_ms: self
                .key_combo_trigger_ms
                .unwrap_or(default.key_combo_trigger_ms),
            cross_hair_alpha: self.cross_hair_alpha.unwrap_or(default.cross_hair_alpha),
            bounding_box_alpha: self
                .bounding_box_alpha
                .unwrap_or(default.bounding_box_alpha),
            display_bounding_box_name: self
                .display_bounding_box_name
                .unwrap_or(default.display_bounding_box_name),
            scroll_thresh: self.scroll_thresh.unwrap_or(default.scroll_thresh),
            start_img_index: self.start_img_index.unwrap_or(default.start_img_index),
            display_cursor_name: self
                .display_cursor_name
                .unwrap_or(default.display_cursor_name),
            save_interval_seconds: self
                .save_interval_seconds
                .unwrap_or(default.save_interval_seconds),
            dynamic_crosshair: self.dynamic_crosshair.unwrap_or(default.dynamic_crosshair),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            key_combo_trigger_ms: 150,
            cross_hair_alpha: 100,
            bounding_box_alpha: 100,
            display_bounding_box_name: true,
            scroll_thresh: 0.0,
            start_img_index: 0,
            display_cursor_name: true,
            save_interval_seconds: NonZeroU32::new(20).unwrap(),
            dynamic_crosshair: false,
        }
    }
}

impl Settings {
    pub(crate) fn from_file() -> std::io::Result<Settings> {
        let f = File::with_options()
            .read(true)
            .write(false)
            .open("settings.json")?;
        let buf_reader = BufReader::new(f);
        let partial: PartialSettings = serde_json::from_reader(buf_reader)
            .expect("invalid json, delete the file if you are fine with defaults we will generate a new one");
        Ok(partial.fill())
    }
}

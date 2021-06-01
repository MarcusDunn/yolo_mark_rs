use std::fs::File;
use std::io::BufReader;

trait Partial<T>
where
    T: Default,
{
    fn fill(&self) -> T;
}

#[derive(serde::Serialize, Debug)]
pub struct Settings {
    pub key_combo_trigger_ms: u128,
}

#[derive(serde::Deserialize)]
struct PartialSettings {
    key_combo_trigger_ms: Option<u128>,
}

impl Partial<Settings> for PartialSettings {
    fn fill(&self) -> Settings {
        let default = Settings::default();
        Settings {
            key_combo_trigger_ms: self
                .key_combo_trigger_ms
                .unwrap_or(default.key_combo_trigger_ms),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            key_combo_trigger_ms: 150,
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

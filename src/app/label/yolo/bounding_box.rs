use crate::app::label::yolo::YoloParseError;
use std::convert::TryFrom;
use std::str::FromStr;

pub struct BoundingBox {
    pub label: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn new(label: u32, x: f64, y: f64, width: f64, height: f64) -> Option<BoundingBox> {
        fn between_1_and_0(num: &f64) -> bool {
            num < &1.0 && num > &0.0
        }

        if [x, y, width, height].iter().all(between_1_and_0) {
            Some(BoundingBox {
                label,
                x,
                y,
                width,
                height,
            })
        } else {
            None
        }
    }
}

impl TryFrom<String> for BoundingBox {
    type Error = YoloParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let [label, x, y, width, height] = value.split(' ').collect::<Vec<_>>().as_slice() {
            Ok(BoundingBox {
                label: u32::from_str(label)?,
                x: f64::from_str(x)?,
                y: f64::from_str(y)?,
                width: f64::from_str(width)?,
                height: f64::from_str(height)?,
            })
        } else {
            Err(YoloParseError::MalformedLine(value))
        }
    }
}

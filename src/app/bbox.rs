use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

use eframe::egui::color::Hsva;
use eframe::egui::{Align2, Color32, Painter, Pos2, Rect, Stroke, TextStyle, Vec2};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug)]
pub enum BBoxError {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    InvalidLine(String),
    InvalidField(String),
}

impl Display for BBoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BBoxError::ParseIntError(err) => std::fmt::Display::fmt(&err, f),
            BBoxError::ParseFloatError(err) => std::fmt::Display::fmt(&err, f),
            BBoxError::InvalidLine(explanation) => write!(f, "InvalidLine {}", explanation),
            BBoxError::InvalidField(explanation) => write!(f, "InvalidField {}", explanation),
        }
    }
}

impl From<ParseFloatError> for BBoxError {
    fn from(err: ParseFloatError) -> Self {
        Self::ParseFloatError(err)
    }
}

impl From<ParseIntError> for BBoxError {
    fn from(err: ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl Error for BBoxError {}

impl TryFrom<&str> for BBox {
    type Error = BBoxError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split(' ').into_iter().collect::<Vec<_>>();
        if let [name, x, y, width, height] = split.as_slice() {
            Ok(BBox::new(
                name.parse()?,
                width.parse()?,
                height.parse()?,
                x.parse()?,
                y.parse()?,
            )?)
        } else {
            Err(BBoxError::InvalidLine(format!(
                "expected 5 values in {}",
                value,
            )))
        }
    }
}

impl Display for BBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            name,
            width,
            height,
            x,
            y,
            ..
        } = self;
        write!(f, "{} {} {} {} {}", name, x, y, width, height)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    no_init_from_fields: (),
    pub color: [u8; 3],
    pub name: usize,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

impl BBox {
    pub fn yolo_format(&self) -> String {
        self.to_string()
    }

    pub fn from_two_points_and_rect(
        name: usize,
        rect: Rect,
        drag_srt: Pos2,
        drag_diff: Pos2,
    ) -> Result<BBox, BBoxError> {
        BBox::from_two_points(
            name,
            drag_srt - rect.min.to_vec2(),
            drag_srt + drag_diff.to_vec2() - rect.min.to_vec2(),
            rect.size(),
        )
    }

    pub(crate) fn from_two_points(
        name: usize,
        Pos2 {
            x: box_x1,
            y: box_y1,
        }: Pos2,
        Pos2 {
            x: box_x2,
            y: box_y2,
        }: Pos2,
        Vec2 { x: img_w, y: img_h }: Vec2,
    ) -> Result<BBox, BBoxError> {
        let abs_x = (box_x1 + box_x2) / 2.0;
        let abs_y = (box_y1 + box_y2) / 2.0;
        let abs_w = (box_x1 - box_x2).abs();
        let abs_h = (box_y1 - box_y2).abs();

        let rel_width = abs_w / img_w;
        let rel_height = abs_h / img_h;
        let rel_x = abs_x / img_w;
        let rel_y = abs_y / img_h;

        BBox::new(name, rel_width, rel_height, rel_x, rel_y)
    }
}

impl BBox {
    pub(crate) fn draw_text(
        &self,
        painter: &mut Painter,
        names: &[String],
        rect: Rect,
        alpha: u8,
        selected: bool,
    ) {
        let rect = painter.text(
            rect.min - Vec2::new(0.0, 20.0),
            Align2::LEFT_TOP,
            &names[self.name],
            TextStyle::Heading,
            if selected {
                Color32::from_white_alpha(alpha)
            } else {
                self.color_w_alpha(alpha)
            },
        );
        if selected {
            painter.rect(rect, 0.0, Color32::BLACK, Stroke::default())
        }
        painter.text(
            rect.min,
            Align2::LEFT_TOP,
            &names[self.name],
            TextStyle::Heading,
            if selected {
                Color32::from_white_alpha(alpha)
            } else {
                self.color_w_alpha(alpha)
            },
        );
    }
}

impl BBox {
    pub(crate) fn draw(&self, painter: &mut Painter, alpha: u8, selected: bool) -> Rect {
        if selected {
            let color = Color32::from_white_alpha(255);
            let rect = self.with_respect_to(painter.clip_rect());
            BBox::draw_colored_box_outline(painter, color, rect, 2.0);
            rect
        } else {
            let color = self.color_w_alpha(alpha);
            let rect = self.with_respect_to(painter.clip_rect());
            BBox::draw_colored_box_outline(painter, color, rect, 1.0);
            rect
        }
    }

    fn color_w_alpha(&self, alpha: u8) -> Color32 {
        let [r, g, b] = self.color;
        Color32::from_rgba_premultiplied(r, g, b, alpha)
    }

    fn draw_colored_box_outline(painter: &mut Painter, color: Color32, rect: Rect, thickness: f32) {
        let top_left = rect.min;
        let top_right = Pos2 {
            x: rect.min.x + rect.width(),
            y: rect.min.y,
        };
        let bot_left = Pos2 {
            x: rect.min.x,
            y: rect.min.y + rect.height(),
        };
        let bot_right = Pos2 {
            x: rect.min.x + rect.width(),
            y: rect.min.y + rect.height(),
        };
        let thicc = Vec2::new(thickness / 2.0, thickness / 2.0);
        painter.rect(
            Rect::from_two_pos(bot_right + thicc, bot_left - thicc),
            1.0,
            color,
            Stroke::default(),
        );
        painter.rect(
            Rect::from_two_pos(top_left + thicc, bot_left - thicc),
            1.0,
            color,
            Stroke::default(),
        );
        painter.rect(
            Rect::from_two_pos(bot_right + thicc, top_right - thicc),
            1.0,
            color,
            Stroke::default(),
        );
        painter.rect(
            Rect::from_two_pos(top_right + thicc, top_left - thicc),
            1.0,
            color,
            Stroke::default(),
        );
    }
    fn with_respect_to(&self, rect: Rect) -> Rect {
        let abs_x = self.x * rect.width() + rect.min.x;
        let abs_y = self.y * rect.height() + rect.min.y;
        let abs_width = self.width * rect.width();
        let abs_height = self.height * rect.height();
        Rect {
            max: Pos2 {
                x: abs_x + (abs_width / 2.0),
                y: abs_y + (abs_height / 2.0),
            },
            min: Pos2 {
                x: abs_x - (abs_width / 2.0),
                y: abs_y - (abs_height / 2.0),
            },
        }
    }
}

impl BBox {
    fn colour(name: usize) -> [u8; 3] {
        let mut rng = ChaCha8Rng::seed_from_u64(name as u64);
        let h = rng.gen::<u8>() as f32 / 255.0;
        Hsva {
            h,
            s: 1.0,
            v: 1.0,
            a: 0 as f32,
        }
        .to_srgb()
    }

    pub fn new(name: usize, width: f32, height: f32, x: f32, y: f32) -> Result<BBox, BBoxError> {
        if !(0.0001..=1.0).contains(&width) {
            Err(BBoxError::InvalidField(format!(
                "width of {} not in [0.0001..=1]",
                width
            )))
        } else if !(0.0001..=1.0).contains(&height) {
            Err(BBoxError::InvalidField(format!(
                "height of {} not in [0.0001..=1]",
                height
            )))
        } else if !(0.0..=1.0).contains(&x) {
            Err(BBoxError::InvalidField(format!(
                "x of {} not in [0..=1]",
                x
            )))
        } else if !(0.0..=1.0).contains(&y) {
            Err(BBoxError::InvalidField(format!(
                "y of {} not in [0..=1]",
                y
            )))
        } else if !(0.0..=1.0).contains(&(x + width / 2.0))
            || !(0.0..=1.0).contains(&(x - width / 2.0))
            || !(0.0..=1.0).contains(&(y + height / 2.0))
            || !(0.0..=1.0).contains(&(y - height / 2.0))
        {
            Err(BBoxError::InvalidField(
                "there's a corner out of bounds!".to_string(),
            ))
        } else {
            Ok(BBox {
                no_init_from_fields: (),
                color: Self::colour(name),
                name,
                width,
                height,
                x,
                y,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn from_line_and_to_line_are_opposite(bbox: BBox) -> bool {
        let and_back_again = BBox::try_from(bbox.to_string().as_str()).unwrap();
        almost_eq(&bbox, &and_back_again)
    }

    fn almost_eq(bbox1: &BBox, bbox2: &BBox) -> bool {
        (bbox1.height - bbox2.height) < 0.000001
            && (bbox1.width - bbox2.width) < 0.000001
            && (bbox1.x - bbox2.x) < 0.000001
            && (bbox1.y - bbox2.y) < 0.000001
            && bbox1.name == bbox2.name
    }

    impl Arbitrary for BBox {
        fn arbitrary(generator: &mut Gen) -> Self {
            let width = u8::arbitrary(generator).saturating_add(1) as f32 / 255.0;
            let height = u8::arbitrary(generator).saturating_add(1) as f32 / 255.0;
            let x = u8::arbitrary(generator) as f32 / 255.0;
            let y = u8::arbitrary(generator) as f32 / 255.0;
            BBox::new(usize::arbitrary(generator), width, height, x, y).unwrap()
        }
    }
}

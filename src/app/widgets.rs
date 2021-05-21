use eframe::egui::{Image, Pos2, Rect, Response, Sense, TextureId, Ui, Vec2, Widget};

pub struct DrawableImage<'a> {
    image: Image,
    boxes: &'a Vec<BBox>,
}

impl<'a> DrawableImage<'a> {
    pub fn new(texture_id: TextureId, size: impl Into<Vec2>, boxes: &'a mut Vec<BBox>) -> Self {
        DrawableImage {
            image: Image::new(texture_id, size),
            boxes,
        }
    }
}

impl Widget for DrawableImage<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.image.size(), Sense::click_and_drag());

        if ui.clip_rect().intersects(rect) {
            let selection = ui.visuals().selection;

            self.image.paint_at(ui, rect);

            for bbox in self.boxes {
                ui.painter().rect_stroke(
                    bbox.with_respect_to(rect.size())
                        .translate(rect.min.to_vec2())
                        .intersect(rect),
                    1.0,
                    selection.stroke,
                );
            }
        }

        response
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BBox {
    relative_x: f64,
    relative_y: f64,
    relative_width: f64,
    relative_height: f64,
}

impl BBox {
    pub fn new(img_size: impl Into<Vec2>, absolute_rect: Rect) -> Result<Self, anyhow::Error> {
        fn check_range(target: f64) -> Result<f64, anyhow::Error> {
            if target.is_normal() && target > 0.0 && target < 1.0 {
                Ok(target)
            } else {
                Err(anyhow::Error::msg(format!(
                    "{} does not fit darkent box requirements",
                    target
                )))
            }
        }

        let Vec2 {
            x: width,
            y: height,
        } = img_size.into();

        let relative_width = check_range((absolute_rect.width() / width) as f64)?;
        let relative_height = check_range((absolute_rect.height() / height) as f64)?;
        let relative_x = check_range((absolute_rect.center().x / width) as f64)?;
        let relative_y = check_range((absolute_rect.center().y / height) as f64)?;

        Ok(BBox {
            relative_height,
            relative_width,
            relative_x,
            relative_y,
        })
    }

    pub fn with_respect_to(self, img_size: Vec2) -> Rect {
        let Vec2 {
            x: width,
            y: height,
        } = img_size;
        let center = Pos2 {
            x: (self.relative_x as f32 * width),
            y: (self.relative_y as f32 * height),
        };
        let size = Vec2 {
            x: (self.relative_width as f32 * width),
            y: (self.relative_height as f32 * height),
        };
        Rect::from_center_size(center, size)
    }
}

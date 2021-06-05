use eframe::egui::Pos2;

pub struct DragStatus {
    pub(crate) drag_start: Option<Pos2>,
    pub(crate) drag_diff: Option<Pos2>,
}

impl DragStatus {
    pub(crate) fn start(&mut self, pos: Pos2) {
        self.drag_start = Some(pos);
        self.drag_diff = Some(Pos2::ZERO);
    }
}

impl DragStatus {
    pub(crate) fn clear(&mut self) {
        self.drag_diff = None;
        self.drag_start = None;
    }
}

impl DragStatus {
    pub(crate) fn empty() -> DragStatus {
        DragStatus {
            drag_start: None,
            drag_diff: None,
        }
    }
}

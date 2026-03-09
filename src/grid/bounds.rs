use core_graphics::display::CGDisplay;

#[derive(Clone, Copy, Debug)]
pub struct GridBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl GridBounds {
    pub fn from_display(display: CGDisplay) -> Self {
        let bounds = display.bounds();
        Self {
            x: bounds.origin.x,
            y: bounds.origin.y,
            width: bounds.size.width,
            height: bounds.size.height,
        }
    }

    pub fn subdivide(&self, row: i32, col: i32) -> Self {
        let cell_width = self.width / 3.0;
        let cell_height = self.height / 3.0;

        Self {
            x: self.x + (col as f64) * cell_width,
            y: self.y + (row as f64) * cell_height,
            width: cell_width,
            height: cell_height,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let target_x = self.x + (self.width / 2.0);
        let target_y = self.y + (self.height / 2.0);
        (target_x.round() as i32, target_y.round() as i32)
    }
}

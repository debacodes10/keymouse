#[derive(Clone, Copy, Debug)]
pub struct GridBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl GridBounds {
    #[cfg(target_os = "macos")]
    pub fn from_display(display: core_graphics::display::CGDisplay) -> Self {
        let bounds = display.bounds();
        Self {
            x: bounds.origin.x,
            y: bounds.origin.y,
            width: bounds.size.width,
            height: bounds.size.height,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn from_rect(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn subdivide(&self, row: i32, col: i32) -> Self {
        let cell_width = self.width / 3.0;
        let cell_height = self.height / 3.0;
        // Grid rows map top-to-bottom (QWE, ASD, ZXC). Quartz global display
        // coordinates use a top-origin Y axis, so row index maps directly.

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

#[cfg(test)]
mod tests {
    use super::GridBounds;

    #[test]
    fn subdivide_preserves_top_to_bottom_row_order() {
        let root = GridBounds {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 300.0,
        };

        let top = root.subdivide(0, 1);
        let middle = root.subdivide(1, 1);
        let bottom = root.subdivide(2, 1);

        assert!(top.y < middle.y);
        assert!(middle.y < bottom.y);
    }
}

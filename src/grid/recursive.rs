use super::bounds::GridBounds;

#[derive(Debug)]
pub struct RecursiveGrid {
    active: bool,
    current_bounds: Option<GridBounds>,
    rendered_bounds: Option<GridBounds>,
}

impl RecursiveGrid {
    pub fn new() -> Self {
        Self {
            active: false,
            current_bounds: None,
            rendered_bounds: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self, full_display_bounds: GridBounds) {
        self.active = true;
        self.current_bounds = Some(full_display_bounds);
        self.render_overlay(full_display_bounds);
    }

    pub fn zoom_into_cell(&mut self, row: i32, col: i32) -> Option<GridBounds> {
        if !self.active {
            return None;
        }

        let current = self.current_bounds?;
        let next = current.subdivide(row, col);
        self.current_bounds = Some(next);
        self.render_overlay(next);
        Some(next)
    }

    pub fn confirm(&mut self) -> Option<GridBounds> {
        if !self.active {
            return None;
        }

        self.active = false;
        self.rendered_bounds = None;
        self.current_bounds.take()
    }

    pub fn cancel(&mut self) {
        self.active = false;
        self.current_bounds = None;
        self.rendered_bounds = None;
    }

    fn render_overlay(&mut self, bounds: GridBounds) {
        // Keep rendering state lightweight and allocation-free in callback paths.
        self.rendered_bounds = Some(bounds);
    }
}

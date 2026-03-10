use super::bounds::GridBounds;

#[derive(Debug)]
pub struct RecursiveGrid {
    active: bool,
    current_bounds: Option<GridBounds>,
    rendered_bounds: Option<GridBounds>,
    depth: usize,
}

impl RecursiveGrid {
    pub fn new() -> Self {
        Self {
            active: false,
            current_bounds: None,
            rendered_bounds: None,
            depth: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self, full_display_bounds: GridBounds) {
        self.active = true;
        self.depth = 0;
        self.current_bounds = Some(full_display_bounds);
        self.render_overlay(full_display_bounds);
    }

    pub fn zoom_into_cell(&mut self, row: i32, col: i32) -> Option<GridBounds> {
        if !self.active {
            return None;
        }

        let current = self.current_bounds?;
        let next = current.subdivide(row, col);
        self.depth += 1;
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
        self.depth = 0;
        self.current_bounds.take()
    }

    pub fn cancel(&mut self) {
        self.active = false;
        self.current_bounds = None;
        self.rendered_bounds = None;
        self.depth = 0;
    }

    pub fn render_state(&self) -> Option<(GridBounds, usize)> {
        if !self.active {
            return None;
        }

        self.rendered_bounds.map(|bounds| (bounds, self.depth))
    }

    fn render_overlay(&mut self, bounds: GridBounds) {
        // Keep rendering state lightweight and allocation-free in callback paths.
        self.rendered_bounds = Some(bounds);
    }
}

#[cfg(test)]
mod tests {
    use super::{GridBounds, RecursiveGrid};

    fn root_bounds() -> GridBounds {
        GridBounds {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 300.0,
        }
    }

    #[test]
    fn start_and_zoom_updates_render_state_and_depth() {
        let mut grid = RecursiveGrid::new();
        grid.start(root_bounds());
        assert!(grid.is_active());
        assert_eq!(grid.render_state().map(|(_, depth)| depth), Some(0));

        let next = grid.zoom_into_cell(0, 0).expect("zoom should succeed");
        let (rendered, depth) = grid.render_state().expect("render state should exist");
        assert_eq!(depth, 1);
        assert_eq!(rendered.x, next.x);
        assert_eq!(rendered.y, next.y);
    }

    #[test]
    fn confirm_returns_current_bounds_and_deactivates_grid() {
        let mut grid = RecursiveGrid::new();
        grid.start(root_bounds());
        let _ = grid.zoom_into_cell(1, 2);

        let confirmed = grid.confirm().expect("confirm should return bounds");
        assert!(!grid.is_active());
        assert!(grid.render_state().is_none());
        assert_eq!(confirmed.width, 100.0);
        assert_eq!(confirmed.height, 100.0);
    }

    #[test]
    fn cancel_resets_all_state() {
        let mut grid = RecursiveGrid::new();
        grid.start(root_bounds());
        let _ = grid.zoom_into_cell(2, 1);
        grid.cancel();

        assert!(!grid.is_active());
        assert!(grid.render_state().is_none());
        assert!(grid.confirm().is_none());
    }
}

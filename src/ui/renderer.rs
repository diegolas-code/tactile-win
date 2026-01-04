//! Grid rendering system for overlay windows
//!
//! Implements grid visualization with letter labels using tiny-skia for high-performance
//! rendering. Separates layout calculation from rendering for better testability.

use std::collections::HashMap;

use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect as SkiaRect, Stroke, Transform};

use crate::domain::core::Rect;
use crate::domain::grid::Grid;
use crate::domain::keyboard::GridCoords;

/// Rendering errors
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Failed to create pixmap for rendering")]
    PixmapCreationFailed,

    #[error("Invalid grid dimensions: {width}x{height}")]
    InvalidGridDimensions { width: i32, height: i32 },

    #[error("Rendering operation failed")]
    RenderingFailed,
}

/// Represents a single line segment for grid rendering
#[derive(Debug, Clone)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub width: f32,
    pub color: Color,
}

/// Represents a letter label position and properties
#[derive(Debug, Clone)]
pub struct LetterPosition {
    pub letter: char,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub color: Color,
    pub cell_rect: SkiaRect,
}

/// Pre-calculated layout for grid rendering
///
/// Separates layout calculation from actual rendering for better testing
/// and performance. This contains all the geometric information needed
/// to render a grid.
#[derive(Debug, Clone)]
pub struct GridLayout {
    /// Grid lines (horizontal and vertical)
    pub lines: Vec<Line>,

    /// Letter positions for keyboard layout
    pub letters: Vec<LetterPosition>,

    /// Overall canvas dimensions
    pub canvas_width: f32,
    pub canvas_height: f32,

    /// Whether this layout is for an active monitor (shows letters)
    pub is_active: bool,
}

impl GridLayout {
    /// Create a grid layout from domain Grid and keyboard layout
    pub fn from_grid(grid: &Grid, canvas_rect: Rect, is_active: bool, dpi_scale: f32) -> Self {
        let mut layout = Self {
            lines: Vec::new(),
            letters: Vec::new(),
            canvas_width: canvas_rect.w as f32,
            canvas_height: canvas_rect.h as f32,
            is_active,
        };

        // Calculate grid lines
        layout.calculate_grid_lines(grid, canvas_rect, dpi_scale);

        // Calculate letter positions if active
        if is_active {
            layout.calculate_letter_positions(grid, canvas_rect, dpi_scale);
        }

        layout
    }

    /// Calculate horizontal and vertical grid lines
    fn calculate_grid_lines(&mut self, grid: &Grid, canvas_rect: Rect, dpi_scale: f32) {
        let line_width = (2.0 * dpi_scale).max(1.0);
        let line_color = Color::from_rgba8(255, 255, 255, 180); // Semi-transparent white

        let (rows, cols) = grid.dimensions();
        let cell_width = (canvas_rect.w as f32) / (cols as f32);
        let cell_height = (canvas_rect.h as f32) / (rows as f32);

        // Vertical lines (between columns)
        for col in 1..cols {
            let x = (col as f32) * cell_width;
            self.lines.push(Line {
                x1: x,
                y1: 0.0,
                x2: x,
                y2: canvas_rect.h as f32,
                width: line_width,
                color: line_color,
            });
        }

        // Horizontal lines (between rows)
        for row in 1..rows {
            let y = (row as f32) * cell_height;
            self.lines.push(Line {
                x1: 0.0,
                y1: y,
                x2: canvas_rect.w as f32,
                y2: y,
                width: line_width,
                color: line_color,
            });
        }
    }

    /// Calculate letter positions for keyboard layout
    fn calculate_letter_positions(&mut self, grid: &Grid, canvas_rect: Rect, dpi_scale: f32) {
        let (rows, cols) = grid.dimensions();
        let cell_width = (canvas_rect.w as f32) / (cols as f32);
        let cell_height = (canvas_rect.h as f32) / (rows as f32);
        let font_size = (32.0 * dpi_scale).max(24.0);
        let letter_color = Color::from_rgba8(255, 255, 255, 255); // Fully opaque white

        // Get all valid grid positions from keyboard layout
        for row in 0..rows {
            for col in 0..cols {
                let coords = GridCoords::new(col, row);
                if let Ok(letter) = grid.key_for_coords(coords) {
                    // Calculate cell center
                    let cell_center_x = ((col as f32) + 0.5) * cell_width;
                    let cell_center_y = ((row as f32) + 0.5) * cell_height;

                    // Calculate cell rectangle for background highlighting
                    let cell_rect = SkiaRect::from_xywh(
                        (col as f32) * cell_width,
                        (row as f32) * cell_height,
                        cell_width,
                        cell_height,
                    )
                    .unwrap();

                    self.letters.push(LetterPosition {
                        letter,
                        x: cell_center_x,
                        y: cell_center_y,
                        font_size,
                        color: letter_color,
                        cell_rect,
                    });
                }
            }
        }
    }
}

/// High-performance grid renderer using tiny-skia
#[derive(Debug)]
pub struct GridRenderer {
    /// Cached font data for text rendering
    font_cache: HashMap<u32, Vec<u8>>, // font_size -> font_data
}

impl GridRenderer {
    /// Create a new grid renderer
    pub fn new() -> Self {
        Self {
            font_cache: HashMap::new(),
        }
    }

    /// Render a grid layout to a pixmap
    pub fn render_layout(&mut self, layout: &GridLayout) -> Result<Pixmap, RendererError> {
        // Create pixmap for rendering
        let mut pixmap = Pixmap::new(layout.canvas_width as u32, layout.canvas_height as u32)
            .ok_or(RendererError::PixmapCreationFailed)?;

        // Clear with transparent background
        pixmap.fill(Color::TRANSPARENT);

        // Render grid lines
        self.render_lines(&mut pixmap, &layout.lines)?;

        // Render letters if active
        if layout.is_active {
            self.render_letters(&mut pixmap, &layout.letters)?;
        }

        Ok(pixmap)
    }

    /// Render grid lines to the pixmap
    fn render_lines(&self, pixmap: &mut Pixmap, lines: &[Line]) -> Result<(), RendererError> {
        for line in lines {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(line.x1, line.y1);
            path_builder.line_to(line.x2, line.y2);

            if let Some(path) = path_builder.finish() {
                let mut paint = Paint::default();
                paint.set_color(line.color);

                let stroke = Stroke {
                    width: line.width,
                    ..Stroke::default()
                };

                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }

        Ok(())
    }

    /// Render letter labels to the pixmap
    fn render_letters(
        &mut self,
        pixmap: &mut Pixmap,
        letters: &[LetterPosition],
    ) -> Result<(), RendererError> {
        for letter_pos in letters {
            self.render_single_letter(pixmap, letter_pos)?;
        }

        Ok(())
    }

    /// Render a single letter at the specified position
    fn render_single_letter(
        &mut self,
        pixmap: &mut Pixmap,
        letter_pos: &LetterPosition,
    ) -> Result<(), RendererError> {
        // For now, render a simple filled circle as a placeholder for the letter
        // In a full implementation, we would use a font rasterizer like rusttype or ab_glyph
        let radius = letter_pos.font_size / 4.0;

        let mut path_builder = PathBuilder::new();
        if SkiaRect::from_xywh(
            letter_pos.x - radius,
            letter_pos.y - radius,
            radius * 2.0,
            radius * 2.0,
        )
        .is_some()
        {
            path_builder.push_circle(letter_pos.x, letter_pos.y, radius);

            if let Some(path) = path_builder.finish() {
                let mut paint = Paint::default();
                paint.set_color(letter_pos.color);

                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );

                // TODO: Add actual text rendering here
                // This would require integrating a font library like rusttype
                // For Phase 3, the circles serve as visual placeholders
            }
        }

        Ok(())
    }

    /// Convert pixmap to Win32 compatible bitmap data
    /// Returns RGBA byte array suitable for Win32 display
    pub fn pixmap_to_rgba(&self, pixmap: &Pixmap) -> Vec<u8> {
        // tiny-skia uses RGBA format, which is compatible with Win32
        pixmap.data().to_vec()
    }

    /// Get pixmap dimensions
    pub fn get_pixmap_size(&self, pixmap: &Pixmap) -> (u32, u32) {
        (pixmap.width(), pixmap.height())
    }
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::keyboard::QwertyLayout;

    #[test]
    fn grid_layout_creation() {
        let grid = Grid::new(
            2,
            3,
            Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
        )
        .unwrap();
        let canvas_rect = Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        };

        let grid_layout = GridLayout::from_grid(&grid, canvas_rect, true, 1.0);

        // Should have vertical and horizontal lines
        assert!(!grid_layout.lines.is_empty(), "Grid should have lines");

        // Should have letters when active
        assert!(
            !grid_layout.letters.is_empty(),
            "Active grid should have letters"
        );

        // Check canvas dimensions
        assert_eq!(grid_layout.canvas_width, 1920.0);
        assert_eq!(grid_layout.canvas_height, 1080.0);
    }

    #[test]
    fn inactive_grid_layout() {
        let grid = Grid::new(
            2,
            3,
            Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
        )
        .unwrap();
        let canvas_rect = Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        };

        let grid_layout = GridLayout::from_grid(&grid, canvas_rect, false, 1.0);

        // Should have lines but no letters when inactive
        assert!(!grid_layout.lines.is_empty(), "Grid should have lines");
        assert!(
            grid_layout.letters.is_empty(),
            "Inactive grid should have no letters"
        );
    }

    #[test]
    fn dpi_scaling() {
        let grid = Grid::new(
            2,
            2,
            Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1080,
            },
        )
        .unwrap();
        let canvas_rect = Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        };

        let normal_layout = GridLayout::from_grid(&grid, canvas_rect, true, 1.0);
        let scaled_layout = GridLayout::from_grid(&grid, canvas_rect, true, 2.0);

        // Line width should scale with DPI
        if let (Some(normal_line), Some(scaled_line)) =
            (normal_layout.lines.first(), scaled_layout.lines.first())
        {
            assert!(
                scaled_line.width > normal_line.width,
                "Scaled lines should be thicker"
            );
        }

        // Font size should scale with DPI
        if let (Some(normal_letter), Some(scaled_letter)) =
            (normal_layout.letters.first(), scaled_layout.letters.first())
        {
            assert!(
                scaled_letter.font_size > normal_letter.font_size,
                "Scaled fonts should be larger"
            );
        }
    }

    #[test]
    fn grid_renderer_creation() {
        let renderer = GridRenderer::new();
        assert!(renderer.font_cache.is_empty());
    }

    #[test]
    fn render_simple_layout() {
        let mut renderer = GridRenderer::new();
        let grid = Grid::new(
            2,
            2,
            Rect {
                x: 0,
                y: 0,
                w: 1000,
                h: 800,
            },
        )
        .unwrap();
        let canvas_rect = Rect {
            x: 0,
            y: 0,
            w: 1000,
            h: 800,
        };

        let grid_layout = GridLayout::from_grid(&grid, canvas_rect, true, 1.0);

        // Should be able to render without error
        let result = renderer.render_layout(&grid_layout);
        assert!(result.is_ok(), "Rendering should succeed");

        if let Ok(pixmap) = result {
            let (width, height) = renderer.get_pixmap_size(&pixmap);
            assert_eq!(width, 1000);
            assert_eq!(height, 800);
        }
    }

    #[test]
    fn pixmap_to_rgba_conversion() {
        let mut renderer = GridRenderer::new();
        let grid = Grid::new(
            2,
            2,
            Rect {
                x: 0,
                y: 0,
                w: 1000,
                h: 800,
            },
        )
        .unwrap();
        let canvas_rect = Rect {
            x: 0,
            y: 0,
            w: 1000,
            h: 800,
        };

        let grid_layout = GridLayout::from_grid(&grid, canvas_rect, false, 1.0); // inactive for simplicity

        if let Ok(pixmap) = renderer.render_layout(&grid_layout) {
            let rgba_data = renderer.pixmap_to_rgba(&pixmap);

            // Should have RGBA data (4 bytes per pixel)
            let expected_size = 1000 * 800 * 4;
            assert_eq!(rgba_data.len(), expected_size);
        }
    }
}

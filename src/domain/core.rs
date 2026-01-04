//! Core domain types and operations
//!
//! This module defines pure domain types that work exclusively with
//! real pixels and have no knowledge of Win32 or DPI concepts.

/// Rectangle in real pixel coordinates
///
/// This is the fundamental building block for all geometric calculations.
/// All coordinates are in real pixels, already DPI-normalized by the platform layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    /// Creates a new rectangle
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// Returns the right edge coordinate
    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    /// Returns the bottom edge coordinate
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    /// Returns true if this rectangle contains the given point
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Returns the intersection of two rectangles, or None if they don't intersect
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if left < right && top < bottom {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }

    /// Returns the bounding box that contains both rectangles
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Rect::new(left, top, right - left, bottom - top)
    }

    /// Returns the area of the rectangle in square pixels
    pub fn area(&self) -> i32 {
        self.w * self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_basic_properties() {
        let rect = Rect::new(10, 20, 100, 50);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.w, 100);
        assert_eq!(rect.h, 50);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 70);
        assert_eq!(rect.area(), 5000);
    }

    #[test]
    fn rect_contains_point() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains_point(15, 15)); // Inside
        assert!(rect.contains_point(10, 10)); // Top-left corner
        assert!(!rect.contains_point(30, 30)); // Outside right-bottom
        assert!(!rect.contains_point(5, 5)); // Outside left-top
    }

    #[test]
    fn rect_intersection() {
        let rect1 = Rect::new(0, 0, 20, 20);
        let rect2 = Rect::new(10, 10, 20, 20);
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rect::new(10, 10, 10, 10));

        // No intersection
        let rect3 = Rect::new(30, 30, 10, 10);
        assert!(rect1.intersection(&rect3).is_none());
    }

    #[test]
    fn rect_union() {
        let rect1 = Rect::new(0, 0, 10, 10);
        let rect2 = Rect::new(20, 20, 10, 10);
        let union = rect1.union(&rect2);
        assert_eq!(union, Rect::new(0, 0, 30, 30));
    }
}

use ::std::{num::TryFromIntError, ops::Add};

use ::sdl3_sys::rect::SDL_FRect;

/// The type of pixel unit.
pub type PointUnit = f32;

/// The rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub position: Point,
    pub size: Size,
}

impl Rect {
    #[must_use]
    pub fn new(x: PointUnit, y: PointUnit, width: PointUnit, height: PointUnit) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Check if a point is in the rectangle.
    #[must_use]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.position.x
            && p.x < self.position.x + self.size.width
            && p.y >= self.position.y
            && p.y < self.position.y + self.size.height
    }

    /// Add padding to the rectangle.
    ///
    /// It can avoid the pixel pollution when rendering.
    #[must_use]
    pub fn inset(&self, padding: PointUnit) -> Self {
        Self::new(
            self.position.x + padding,
            self.position.y + padding,
            self.size.width - padding * 2.0,
            self.size.height - padding * 2.0,
        )
    }
}

impl From<(Point, Size)> for Rect {
    fn from((position, size): (Point, Size)) -> Self {
        Self { position, size }
    }
}

impl From<SDL_FRect> for Rect {
    fn from(value: SDL_FRect) -> Self {
        Self {
            position: Point::new(value.x, value.y),
            size: Size::new(value.w, value.h),
        }
    }
}

impl From<Rect> for SDL_FRect {
    fn from(value: Rect) -> Self {
        SDL_FRect {
            x: value.position.x,
            y: value.position.y,
            w: value.size.width,
            h: value.size.height,
        }
    }
}

impl From<Rect> for sdl3_sys::rect::SDL_Rect {
    fn from(value: Rect) -> Self {
        sdl3_sys::rect::SDL_Rect {
            x: value.position.x as i32,
            y: value.position.y as i32,
            w: value.size.width as i32,
            h: value.size.height as i32,
        }
    }
}

impl From<sdl3_sys::rect::SDL_Rect> for Rect {
    fn from(value: sdl3_sys::rect::SDL_Rect) -> Self {
        Self {
            position: Point::new(value.x as PointUnit, value.y as PointUnit),
            size: Size::new(value.w as PointUnit, value.h as PointUnit),
        }
    }
}

impl AsRef<Point> for Rect {
    fn as_ref(&self) -> &Point {
        &self.position
    }
}

impl AsRef<Size> for Rect {
    fn as_ref(&self) -> &Size {
        &self.size
    }
}

/// The point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: PointUnit,
    pub y: PointUnit,
}

impl Point {
    pub fn new(x: PointUnit, y: PointUnit) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: PointUnit,
    pub height: PointUnit,
}

impl Size {
    pub fn new(width: PointUnit, height: PointUnit) -> Self {
        Self { width, height }
    }

    /// Calculate the area of the rectangle.
    #[must_use]
    pub fn area(&self) -> PointUnit {
        self.width * self.height
    }

    #[must_use]
    pub fn outset(&self, padding: PointUnit) -> Self {
        Self::new(self.width + padding * 2.0, self.height + padding * 2.0)
    }

    #[must_use]
    pub fn inset(&self, padding: PointUnit) -> Self {
        Self::new(self.width - padding * 2.0, self.height - padding * 2.0)
    }

    #[must_use]
    pub fn max_dimension(&self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }
}

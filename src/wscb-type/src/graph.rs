use ::std::num::TryFromIntError;

/// The type of pixel unit.
pub type PointUnit = u32;

/// The rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            && p.x < self.position.x.saturating_add(self.size.width)
            && p.y >= self.position.y
            && p.y < self.position.y.saturating_add(self.size.height)
    }

    /// Add padding to the rectangle.
    ///
    /// It can avoid the pixel pollution when rendering.
    #[must_use]
    pub fn inset(&self, padding: PointUnit) -> Self {
        Self::new(
            self.position.x.saturating_add(padding),
            self.position.y.saturating_add(padding),
            self.size.width.saturating_sub(padding.saturating_mul(2)),
            self.size.height.saturating_sub(padding.saturating_mul(2)),
        )
    }
}

impl From<(Point, Size)> for Rect {
    fn from((position, size): (Point, Size)) -> Self {
        Self { position, size }
    }
}

impl TryFrom<sdl3_sys::rect::SDL_Rect> for Rect {
    type Error = TryFromIntError;

    fn try_from(value: sdl3_sys::rect::SDL_Rect) -> Result<Self, Self::Error> {
        Ok(Self {
            position: Point::new(value.x.try_into()?, value.y.try_into()?),
            size: Size::new(value.w.try_into()?, value.h.try_into()?),
        })
    }
}

impl TryFrom<Rect> for sdl3_sys::rect::SDL_Rect {
    type Error = TryFromIntError;

    fn try_from(value: Rect) -> Result<Self, Self::Error> {
        Ok(sdl3_sys::rect::SDL_Rect {
            x: value.position.x.try_into()?,
            y: value.position.y.try_into()?,
            w: value.size.width.try_into()?,
            h: value.size.height.try_into()?,
        })
    }
}

impl From<Rect> for sdl3_sys::rect::SDL_FRect {
    fn from(value: Rect) -> Self {
        sdl3_sys::rect::SDL_FRect {
            x: value.position.x as f32,
            y: value.position.y as f32,
            w: value.size.width as f32,
            h: value.size.height as f32,
        }
    }
}

impl From<sdl3_sys::rect::SDL_FRect> for Rect {
    fn from(value: sdl3_sys::rect::SDL_FRect) -> Self {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: PointUnit,
    pub y: PointUnit,
}

impl Point {
    pub fn new(x: PointUnit, y: PointUnit) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        self.width.saturating_mul(self.height)
    }

    #[must_use]
    pub fn outset(&self, padding: PointUnit) -> Self {
        Self::new(
            self.width.saturating_add(padding.saturating_mul(2)),
            self.height.saturating_add(padding.saturating_mul(2)),
        )
    }

    #[must_use]
    pub fn inset(&self, padding: PointUnit) -> Self {
        Self::new(
            self.width.saturating_sub(padding.saturating_mul(2)),
            self.height.saturating_sub(padding.saturating_mul(2)),
        )
    }

    #[must_use]
    pub fn max_dimension(&self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }
}

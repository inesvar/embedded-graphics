//! The rectangle primitive.

mod points;

use crate::{
    geometry::{AnchorPoint, AnchorX, AnchorY, Dimensions, Point, Size},
    primitives::PointsIter,
};
use az::SaturatingAs;
use core::{
    cmp::min,
    ops::{Range, RangeInclusive},
};
pub use points::Points;

/// Rectangle primitive
///
/// # Examples
///
/// ## Create some rectangles with different styles
///
/// ```rust
/// use embedded_graphics::{
///     pixelcolor::Rgb565, prelude::*, primitives::{Rectangle, PrimitiveStyleBuilder},
/// };
/// # use embedded_graphics::mock_display::MockDisplay;
/// # let mut display = MockDisplay::default();
///
/// // Rectangle with red 3 pixel wide stroke and green fill with the top left corner at (30, 20) and
/// // a size of (10, 15)
/// let style = PrimitiveStyleBuilder::new()
///     .stroke_color(Rgb565::RED)
///     .stroke_width(3)
///     .fill_color(Rgb565::GREEN)
///     .build();
///
/// Rectangle::new(Point::new(30, 20), Size::new(10, 15))
///     .into_styled(style)
///     .draw(&mut display)?;
///
/// // Rectangle with translation applied
/// Rectangle::new(Point::new(30, 20), Size::new(10, 15))
///     .translate(Point::new(-20, -10))
///     .into_styled(style)
///     .draw(&mut display)?;
/// # Ok::<(), core::convert::Infallible>(())
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct Rectangle {
    /// Top left point of the rectangle.
    pub top_left: Point,

    /// Size of the rectangle.
    pub size: Size,
}

impl Dimensions for Rectangle {
    fn bounding_box(&self) -> Rectangle {
        *self
    }
}

impl PointsIter for Rectangle {
    type Iter = Points;

    fn points(&self) -> Self::Iter {
        Points::new(self)
    }
}

/// Returns the center offset.
///
/// The center offset is defined as the offset between the top left corner and
/// the center point of a rectangle with the given size.
const fn center_offset(size: Size) -> Size {
    size.saturating_sub(Size::new_equal(1)).div_u32(2)
}

impl Rectangle {
    /// Creates a new rectangle from the top left point and the size.
    pub const fn new(top_left: Point, size: Size) -> Self {
        Rectangle { top_left, size }
    }

    /// Creates a new rectangle with the given size and the top left corner at the origin.
    pub const fn new_at_origin(size: Size) -> Self {
        Rectangle {
            top_left: Point::zero(),
            size,
        }
    }

    /// Creates a new rectangle from two corners.
    pub fn with_corners(corner_1: Point, corner_2: Point) -> Self {
        let left = min(corner_1.x, corner_2.x);
        let top = min(corner_1.y, corner_2.y);

        Rectangle {
            top_left: Point::new(left, top),
            size: Size::from_bounding_box(corner_1, corner_2),
        }
    }

    /// Creates a new rectangle from the center point and the size.
    ///
    /// For rectangles with even width and/or height the top left corner doesn't
    /// align with the pixel grid. Because of this the coordinates of the top left
    /// corner will be rounded up to the nearest integer coordinate.
    pub const fn with_center(center: Point, size: Size) -> Self {
        Rectangle {
            top_left: center.sub_size(center_offset(size)),
            size,
        }
    }

    /// Returns a zero sized rectangle.
    pub const fn zero() -> Rectangle {
        Rectangle::new(Point::zero(), Size::zero())
    }

    /// Returns the center of this rectangle.
    ///
    /// For rectangles with even width and/or height the returned value is rounded down
    /// to the nearest integer coordinate.
    pub fn center(&self) -> Point {
        self.top_left + center_offset(self.size)
    }

    /// Returns the bottom right corner of this rectangle.
    ///
    /// Because the smallest rectangle that can be represented by its corners
    /// has a size of 1 x 1 pixels, this function returns `None` if the width or
    /// height of the rectangle is zero.
    pub fn bottom_right(&self) -> Option<Point> {
        if self.size.width > 0 && self.size.height > 0 {
            Some(self.top_left + self.size - Point::new(1, 1))
        } else {
            None
        }
    }

    /// Return whether the rectangle contains a given point.
    pub fn contains(&self, point: Point) -> bool {
        if point.x >= self.top_left.x && point.y >= self.top_left.y {
            self.bottom_right()
                .is_some_and(|bottom_right| point.x <= bottom_right.x && point.y <= bottom_right.y)
        } else {
            false
        }
    }

    /// Returns a new `Rectangle` containing the intersection of `self` and `other`.
    ///
    /// If no intersection is present, this method will return a zero sized rectangle.
    ///
    /// # Examples
    ///
    /// ## Intersection
    ///
    /// This example draws two rectangles to a mock display using the `.` character, along with
    /// their intersection shown with `#` characters.
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    ///     primitives::{Rectangle, PrimitiveStyle},
    /// };
    ///
    /// let mut display = MockDisplay::new();
    /// # display.set_allow_overdraw(true);
    ///
    /// let rect1 = Rectangle::new(Point::zero(), Size::new(7, 8));
    /// let rect2 = Rectangle::new(Point::new(2, 3), Size::new(10, 7));
    ///
    /// let intersection = rect1.intersection(&rect2);
    ///
    /// rect1
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
    ///     .draw(&mut display)?;
    ///
    /// rect2
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
    ///     .draw(&mut display)?;
    ///
    /// intersection
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    ///     .draw(&mut display)?;
    ///
    /// display.assert_pattern(&[
    ///     ".......     ",
    ///     ".     .     ",
    ///     ".     .     ",
    ///     ". #####.....",
    ///     ". #   #    .",
    ///     ". #   #    .",
    ///     ". #   #    .",
    ///     "..#####    .",
    ///     "  .        .",
    ///     "  ..........",
    /// ]);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    ///
    /// ## No intersection
    ///
    /// This example creates two rectangles with no intersection between them. In this case,
    /// `intersection` returns a zero-sized rectangle.
    ///
    /// ```rust
    /// use embedded_graphics::{prelude::*, primitives::{Rectangle, PrimitiveStyle}};
    ///
    /// let rect1 = Rectangle::new(Point::zero(), Size::new(7, 8));
    /// let rect2 = Rectangle::new(Point::new(10, 15), Size::new(10, 7));
    ///
    /// let intersection = rect1.intersection(&rect2);
    ///
    /// assert!(intersection.is_zero_sized());
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    pub fn intersection(&self, other: &Rectangle) -> Rectangle {
        match (other.bottom_right(), self.bottom_right()) {
            (Some(other_bottom_right), Some(self_bottom_right)) => {
                if overlaps(
                    self.top_left.x..=self_bottom_right.x,
                    other.top_left.x..=other_bottom_right.x,
                ) && overlaps(
                    self.top_left.y..=self_bottom_right.y,
                    other.top_left.y..=other_bottom_right.y,
                ) {
                    return Rectangle::with_corners(
                        self.top_left.component_max(other.top_left),
                        self_bottom_right.component_min(other_bottom_right),
                    );
                }
            }
            (Some(_other_bottom_right), None) => {
                // Check if zero sized self is inside other
                if other.contains(self.top_left) {
                    return *self;
                }
            }
            (None, Some(_self_bottom_right)) => {
                // Check if zero sized other is inside self
                if self.contains(other.top_left) {
                    return *other;
                }
            }
            (None, None) => (),
        };

        // No overlap present
        Rectangle::zero()
    }

    /// Returns the minimum sized `Rectangle` that envelopes both `self` and `other`.
    ///
    /// For the purpose of this method, [`Rectangle`]s will be treated as having a minimum
    /// size of `1` along any specific axis. For example, an zero sized [`Rectangle`] will
    /// be treated equivalent to 1x1 [`Rectangle`]. This is done so that the [`Rectangle`] is
    /// represented within the resulting bounding box, even if its [`Size`] would not
    /// result in it being drawn.
    ///
    /// # Examples
    ///
    /// This example draws two rectangles to a mock display using the `.` character, along with
    /// their enveloping rectangle shown with `#` characters.
    ///
    /// ```rust
    /// use embedded_graphics::{
    ///     mock_display::MockDisplay, pixelcolor::BinaryColor, prelude::*,
    ///     primitives::{Rectangle, PrimitiveStyle},
    /// };
    ///
    /// let mut display = MockDisplay::new();
    /// # display.set_allow_overdraw(true);
    ///
    /// let rect1 = Rectangle::new(Point::zero(), Size::new(7, 8));
    /// let rect2 = Rectangle::new(Point::new(2, 3), Size::new(10, 7));
    ///
    /// let envelope = rect1.envelope(&rect2);
    ///
    /// rect1
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
    ///     .draw(&mut display)?;
    ///
    /// rect2
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
    ///     .draw(&mut display)?;
    ///
    /// envelope
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    ///     .draw(&mut display)?;
    ///
    /// display.assert_pattern(&[
    ///     "############",
    ///     "#     .    #",
    ///     "#     .    #",
    ///     "# .........#",
    ///     "# .   .    #",
    ///     "# .   .    #",
    ///     "# .   .    #",
    ///     "#......    #",
    ///     "# .        #",
    ///     "############",
    /// ]);
    /// # Ok::<(), core::convert::Infallible>(())
    /// ```
    pub fn envelope(&self, other: &Rectangle) -> Rectangle {
        let top_left = self.top_left.component_min(other.top_left);
        let bottom_right = self
            .anchor_point(AnchorPoint::BottomRight)
            .component_max(other.anchor_point(AnchorPoint::BottomRight));

        Rectangle::with_corners(top_left, bottom_right)
    }

    /// Returns a resized copy of this rectangle.
    ///
    /// The rectangle is resized relative to the given anchor point.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorPoint,
    /// };
    ///
    /// let rect = Rectangle::new(Point::new(20, 20), Size::new(10, 20));
    /// let resized = rect.resized(Size::new(20, 10), AnchorPoint::Center);
    ///
    /// assert_eq!(
    ///     resized,
    ///     Rectangle::new(Point::new(15, 25), Size::new(20, 10))
    /// );
    /// ```
    pub fn resized(&self, size: Size, anchor_point: AnchorPoint) -> Self {
        let mut resized = *self;
        resized.resize_width_mut(size.width, anchor_point.x());
        resized.resize_height_mut(size.height, anchor_point.y());

        resized
    }

    /// Returns a new rectangle with the given width, resized relative to the given anchor edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorX,
    /// };
    ///
    /// let rect = Rectangle::new(Point::new(20, 20), Size::new(10, 20));
    /// let resized = rect.resized_width(20, AnchorX::Center);
    ///
    /// assert_eq!(
    ///     resized,
    ///     Rectangle::new(Point::new(15, 20), Size::new(20, 20))
    /// );
    /// ```
    pub fn resized_width(&self, width: u32, anchor_x: AnchorX) -> Self {
        let mut resized = *self;
        resized.resize_width_mut(width, anchor_x);

        resized
    }

    /// Returns a new rectangle with the given height, resized relative to the given anchor edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorY,
    /// };
    ///
    /// let rect = Rectangle::new(Point::new(20, 20), Size::new(10, 20));
    /// let resized = rect.resized_height(10, AnchorY::Center);
    ///
    /// assert_eq!(
    ///     resized,
    ///     Rectangle::new(Point::new(20, 25), Size::new(10, 10))
    /// );
    /// ```
    pub fn resized_height(&self, height: u32, anchor_y: AnchorY) -> Self {
        let mut resized = *self;
        resized.resize_height_mut(height, anchor_y);

        resized
    }

    fn resize_width_mut(&mut self, width: u32, anchor_x: AnchorX) {
        // Assume size = 1 for zero sized dimensions.
        let delta =
            self.size.width.saturating_as::<i32>().max(1) - width.saturating_as::<i32>().max(1);

        self.top_left.x += match anchor_x {
            AnchorX::Left => 0,
            AnchorX::Center => delta / 2,
            AnchorX::Right => delta,
        };
        self.size.width = width;
    }

    fn resize_height_mut(&mut self, height: u32, anchor_y: AnchorY) {
        // Assume size = 1 for zero sized dimensions.
        let delta =
            self.size.height.saturating_as::<i32>().max(1) - height.saturating_as::<i32>().max(1);

        self.top_left.y += match anchor_y {
            AnchorY::Top => 0,
            AnchorY::Center => delta / 2,
            AnchorY::Bottom => delta,
        };
        self.size.height = height;
    }

    /// Offset the rectangle by a given value.
    ///
    /// Negative values will shrink the rectangle.
    pub fn offset(&self, offset: i32) -> Self {
        let size = if offset >= 0 {
            self.size.saturating_add(Size::new_equal(offset as u32 * 2))
        } else {
            self.size
                .saturating_sub(Size::new_equal((-offset) as u32 * 2))
        };

        Self::with_center(self.center(), size)
    }

    /// Returns an anchor point.
    ///
    /// # Examples
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorPoint,
    /// };
    ///
    /// let mut rect = Rectangle::new(Point::new(20, 20), Size::new(11, 21));
    ///
    /// assert_eq!(rect.anchor_point(AnchorPoint::TopLeft), Point::new(20, 20));
    /// assert_eq!(
    ///     rect.anchor_point(AnchorPoint::BottomCenter),
    ///     Point::new(25, 40)
    /// );
    /// ```
    pub fn anchor_point(&self, anchor_point: AnchorPoint) -> Point {
        Point::new(
            self.anchor_x(anchor_point.x()),
            self.anchor_y(anchor_point.y()),
        )
    }

    /// Returns the X coordinate of a given anchor edge of the rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorX,
    /// };
    ///
    /// let mut rect = Rectangle::new(Point::new(20, 20), Size::new(11, 21));
    ///
    /// assert_eq!(rect.anchor_x(AnchorX::Left), 20);
    /// assert_eq!(rect.anchor_x(AnchorX::Center), 25);
    /// ```
    pub fn anchor_x(&self, anchor_x: AnchorX) -> i32 {
        // Assume size = 1 for zero sized dimensions.
        let delta = self.size.width.saturating_as::<i32>().max(1) - 1;

        self.top_left.x
            + match anchor_x {
                AnchorX::Left => 0,
                AnchorX::Center => delta / 2,
                AnchorX::Right => delta,
            }
    }

    /// Returns the Y coordinate of a given anchor edge of the rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{
    ///     prelude::*,
    ///     primitives::rectangle::Rectangle,
    ///     geometry::AnchorY,
    /// };
    ///
    /// let mut rect = Rectangle::new(Point::new(20, 20), Size::new(11, 21));
    ///
    /// assert_eq!(rect.anchor_y(AnchorY::Top), 20);
    /// assert_eq!(rect.anchor_y(AnchorY::Bottom), 40);
    /// ```
    pub fn anchor_y(&self, anchor_y: AnchorY) -> i32 {
        // Assume size = 1 for zero sized dimensions.
        let delta = self.size.height.saturating_as::<i32>().max(1) - 1;

        self.top_left.y
            + match anchor_y {
                AnchorY::Top => 0,
                AnchorY::Center => delta / 2,
                AnchorY::Bottom => delta,
            }
    }

    /// Returns the range of Y coordinates in this rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{prelude::*, primitives::Rectangle};
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::new(3, 4));
    /// assert_eq!(rect.rows(), 20..24);
    /// ```
    ///
    /// By combining this method with [`columns`] it is possible to iterate over all pixels inside
    /// the rectangle. This can be more flexible than using the [`points`] iterator, for example,
    /// if a different iteration order is required or some operations should be called once per row.
    ///
    /// ```
    /// use embedded_graphics::{prelude::*, primitives::Rectangle};
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::new(3, 4));
    ///
    /// // Iterate over the y coordinates of the rows in reverse order.
    /// for y in rect.rows().rev() {
    ///     for x in rect.columns() {
    ///         // use x, y coordinates
    ///     }
    /// }
    /// ```
    ///
    /// [`columns`]: Rectangle::columns()
    /// [`points`]: super::PointsIter::points
    pub fn rows(&self) -> Range<i32> {
        self.top_left.y
            ..self
                .top_left
                .y
                .saturating_add(self.size.height.saturating_as())
    }

    /// Returns the range of X coordinates in this rectangle.
    ///
    /// # Examples
    ///
    /// ```
    /// use embedded_graphics::{prelude::*, primitives::Rectangle};
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::new(3, 4));
    ///
    /// assert_eq!(rect.columns(), 10..13);
    /// ```
    ///
    /// By combining this method with [`rows`] it is possible to iterator over all pixels inside
    /// the rectangle. This can be more flexible than using the [`points`] iterator, for example,
    /// if a different iteration order is required or some operations should be called once per row.
    ///
    /// ```
    /// use embedded_graphics::{prelude::*, primitives::Rectangle};
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::new(3, 4));
    ///
    /// // Iterate over all points starting from the top right corner and advancing downwards.
    /// for x in rect.columns().rev() {
    ///     for y in rect.rows() {
    ///         // use x, y coordinates
    ///     }
    /// }
    /// ```
    ///
    /// [`rows`]: Rectangle::rows()
    /// [`points`]: super::PointsIter::points
    pub fn columns(&self) -> Range<i32> {
        self.top_left.x
            ..self
                .top_left
                .x
                .saturating_add(self.size.width.saturating_as())
    }

    /// Returns `true` is the rectangle is zero sized.
    ///
    /// A rectangle is zero sized if the width or height are zero.
    ///
    /// # Examples
    /// ```
    /// use embedded_graphics::{prelude::*, primitives::Rectangle};
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::new(10, 20));
    /// assert_eq!(rect.is_zero_sized(), false);
    ///
    /// let rect = Rectangle::new(Point::new(10, 20), Size::zero());
    /// assert_eq!(rect.is_zero_sized(), true);
    /// ```
    pub const fn is_zero_sized(&self) -> bool {
        self.size.height == 0 || self.size.width == 0
    }
}

/// Checks if the two ranges overlap.
fn overlaps(first: RangeInclusive<i32>, second: RangeInclusive<i32>) -> bool {
    second.contains(first.start())
        || second.contains(first.end())
        || first.start() < second.start() && first.end() > second.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Dimensions, Point, Size};

    #[test]
    fn dimensions() {
        let rect = Rectangle::new(Point::new(5, 10), Size::new(10, 20));

        assert_eq!(
            rect.bounding_box(),
            Rectangle::new(Point::new(5, 10), Size::new(10, 20))
        );
    }

    #[test]
    fn center() {
        let odd = Rectangle::new(Point::new(10, 20), Size::new(5, 7));
        assert_eq!(odd.center(), Point::new(12, 23));

        let even = Rectangle::new(Point::new(20, 30), Size::new(4, 8));
        assert_eq!(even.center(), Point::new(21, 33));
    }

    #[test]
    fn bottom_right() {
        let zero = Rectangle::new(Point::new(10, 20), Size::zero());
        assert_eq!(zero.bottom_right(), None);

        let odd = Rectangle::new(Point::new(10, 20), Size::new(5, 7));
        assert_eq!(odd.bottom_right(), Some(Point::new(14, 26)));

        let even = Rectangle::new(Point::new(20, 30), Size::new(4, 8));
        assert_eq!(even.bottom_right(), Some(Point::new(23, 37)));
    }

    #[test]
    fn rectangle_envelope() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(20, 30));
        let rect2 = Rectangle::new(Point::new_equal(20), Size::new(30, 40));

        assert_eq!(
            rect1.envelope(&rect2),
            Rectangle::new(Point::new_equal(10), Size::new(40, 50))
        );
    }

    #[test]
    fn rectangle_envelope_with_zero_size() {
        let rect1 = Rectangle::new(Point::new_equal(2), Size::new(5, 5));
        let rect2 = Rectangle::new(Point::new_equal(11), Size::zero());

        assert_eq!(
            rect1.envelope(&rect2),
            Rectangle::new(Point::new_equal(2), Size::new_equal(10))
        );
    }

    #[test]
    fn rectangle_envelope_with_zero_along_axis() {
        let rect1 = Rectangle::new(Point::new_equal(2), Size::new(5, 0));
        let rect2 = Rectangle::new(Point::new_equal(3), Size::new(0, 6));

        assert_eq!(
            rect1.envelope(&rect2),
            Rectangle::new(Point::new_equal(2), Size::new(5, 7))
        );
    }

    #[test]
    fn rectangle_envelope_zero_size_treated_as_one() {
        let zero_sized = Rectangle::new(Point::new_equal(2), Size::zero());
        assert_eq!(
            zero_sized.envelope(&zero_sized),
            Rectangle::new(Point::new_equal(2), Size::new_equal(1))
        )
    }

    #[test]
    fn rectangle_envelope_all_zero_size() {
        let rect1 = Rectangle::new(Point::new_equal(2), Size::zero());
        let rect2 = Rectangle::new(Point::new_equal(11), Size::zero());

        assert_eq!(
            rect1.envelope(&rect2),
            Rectangle::new(Point::new_equal(2), Size::new_equal(10))
        );
    }

    #[test]
    fn rectangle_no_envelope() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(10, 10));
        let rect2 = Rectangle::new(Point::new_equal(30), Size::new(10, 10));

        assert_eq!(
            rect1.envelope(&rect2),
            Rectangle::new(Point::new_equal(10), Size::new(30, 30))
        );
    }

    #[test]
    fn rectangle_fully_envelope() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(30, 30));
        let rect2 = Rectangle::new(Point::new_equal(20), Size::new(10, 10));

        assert_eq!(rect1.envelope(&rect2), rect1);
    }

    #[test]
    fn rectangle_intersection() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(20, 30));
        let rect2 = Rectangle::new(Point::new_equal(25), Size::new(30, 40));

        assert_eq!(
            rect1.intersection(&rect2),
            Rectangle::new(Point::new_equal(25), Size::new(5, 15))
        );
    }

    #[test]
    fn rectangle_no_intersection() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(20, 30));
        let rect2 = Rectangle::new(Point::new_equal(35), Size::new(30, 40));

        assert_eq!(
            rect1.intersection(&rect2),
            Rectangle::new(Point::zero(), Size::zero())
        );
    }

    #[test]
    fn rectangle_complete_intersection() {
        let rect1 = Rectangle::new(Point::new_equal(10), Size::new(20, 30));
        let rect2 = rect1;

        assert_eq!(rect1.intersection(&rect2), rect1);
    }

    #[test]
    fn rectangle_contained_intersection() {
        let rect1 = Rectangle::with_corners(Point::new_equal(10), Point::new(20, 30));
        let rect2 = Rectangle::with_corners(Point::new_equal(5), Point::new(30, 40));

        assert_eq!(rect1.intersection(&rect2), rect1);
    }

    #[test]
    fn zero_sized_intersection() {
        let rect1 = Rectangle::new(Point::new(1, 2), Size::new(0, 0));
        let rect2 = Rectangle::new(Point::new(-10, -10), Size::new(20, 20));

        assert_eq!(rect1.intersection(&rect2), rect1);

        let rect1 = Rectangle::new(Point::new(-10, -10), Size::new(20, 20));
        let rect2 = Rectangle::new(Point::new(2, 3), Size::new(0, 0));

        assert_eq!(rect1.intersection(&rect2), rect2);
    }

    /// Test for issue #452
    ///
    /// Rectangles can intersect even if no corner of any rectangle is contained inside the other
    /// rectangle.
    ///
    /// Example:
    ///
    ///     ****
    ///     *  *
    /// ############
    /// #   *  *   #
    /// #   *  *   #
    /// ############
    ///     *  *
    ///     ****
    #[test]
    fn issue_452_broken_intersection_check() {
        let rect1 = Rectangle::new(Point::new(50, 0), Size::new(75, 200));
        let rect2 = Rectangle::new(Point::new(0, 75), Size::new(200, 50));

        let expected = Rectangle::new(Point::new(50, 75), Size::new(75, 50));

        assert_eq!(rect1.intersection(&rect2), expected);
        assert_eq!(rect2.intersection(&rect1), expected);
    }

    #[test]
    fn offset() {
        let center = Point::new(10, 20);
        let rect = Rectangle::with_center(center, Size::new(3, 4));

        assert_eq!(rect.offset(0), rect);

        assert_eq!(
            rect.offset(1),
            Rectangle::with_center(center, Size::new(5, 6))
        );
        assert_eq!(
            rect.offset(2),
            Rectangle::with_center(center, Size::new(7, 8))
        );

        assert_eq!(
            rect.offset(-1),
            Rectangle::with_center(center, Size::new(1, 2))
        );
        assert_eq!(
            rect.offset(-2),
            Rectangle::with_center(center, Size::new(0, 0))
        );
        assert_eq!(
            rect.offset(-3),
            Rectangle::with_center(center, Size::new(0, 0))
        );
    }

    fn test_resized(rect: Rectangle, target_size: Size, tests: &[(AnchorPoint, Point)]) {
        for &(anchor_point, expected_top_left) in tests {
            let resized = rect.resized(target_size, anchor_point);
            let expected = Rectangle::new(expected_top_left, target_size);

            assert_eq!(resized, expected, "{:?}", anchor_point);

            let resized_x = rect.resized_width(target_size.width, anchor_point.x());
            assert_eq!(
                resized_x.top_left,
                Point::new(resized.top_left.x, rect.top_left.y)
            );
            assert_eq!(
                resized_x.size,
                Size::new(resized.size.width, rect.size.height)
            );

            let resized_y = rect.resized_height(target_size.height, anchor_point.y());
            assert_eq!(
                resized_y.top_left,
                Point::new(rect.top_left.x, resized.top_left.y)
            );
            assert_eq!(
                resized_y.size,
                Size::new(rect.size.width, resized.size.height)
            );
        }
    }

    #[test]
    fn resized_smaller() {
        test_resized(
            Rectangle::new(Point::new(10, 20), Size::new(30, 40)),
            Size::new(10, 20),
            &[
                (AnchorPoint::TopLeft, Point::new(10, 20)),
                (AnchorPoint::TopCenter, Point::new(20, 20)),
                (AnchorPoint::TopRight, Point::new(30, 20)),
                (AnchorPoint::CenterLeft, Point::new(10, 30)),
                (AnchorPoint::Center, Point::new(20, 30)),
                (AnchorPoint::CenterRight, Point::new(30, 30)),
                (AnchorPoint::BottomLeft, Point::new(10, 40)),
                (AnchorPoint::BottomCenter, Point::new(20, 40)),
                (AnchorPoint::BottomRight, Point::new(30, 40)),
            ],
        );
    }

    #[test]
    fn resized_larger() {
        test_resized(
            Rectangle::new(Point::new(10, 20), Size::new(30, 40)),
            Size::new(40, 50),
            &[
                (AnchorPoint::TopLeft, Point::new(10, 20)),
                (AnchorPoint::TopCenter, Point::new(5, 20)),
                (AnchorPoint::TopRight, Point::new(0, 20)),
                (AnchorPoint::CenterLeft, Point::new(10, 15)),
                (AnchorPoint::Center, Point::new(5, 15)),
                (AnchorPoint::CenterRight, Point::new(0, 15)),
                (AnchorPoint::BottomLeft, Point::new(10, 10)),
                (AnchorPoint::BottomCenter, Point::new(5, 10)),
                (AnchorPoint::BottomRight, Point::new(0, 10)),
            ],
        );
    }

    #[test]
    fn resized_zero_sized() {
        test_resized(
            Rectangle::new(Point::new(10, 20), Size::zero()),
            Size::new(5, 7),
            &[
                (AnchorPoint::TopLeft, Point::new(10, 20)),
                (AnchorPoint::TopCenter, Point::new(8, 20)),
                (AnchorPoint::TopRight, Point::new(6, 20)),
                (AnchorPoint::CenterLeft, Point::new(10, 17)),
                (AnchorPoint::Center, Point::new(8, 17)),
                (AnchorPoint::CenterRight, Point::new(6, 17)),
                (AnchorPoint::BottomLeft, Point::new(10, 14)),
                (AnchorPoint::BottomCenter, Point::new(8, 14)),
                (AnchorPoint::BottomRight, Point::new(6, 14)),
            ],
        );
    }

    #[test]
    fn resized_to_zero_sized() {
        test_resized(
            Rectangle::new(Point::new(10, 20), Size::new(21, 31)),
            Size::zero(),
            &[
                (AnchorPoint::TopLeft, Point::new(10, 20)),
                (AnchorPoint::TopCenter, Point::new(20, 20)),
                (AnchorPoint::TopRight, Point::new(30, 20)),
                (AnchorPoint::CenterLeft, Point::new(10, 35)),
                (AnchorPoint::Center, Point::new(20, 35)),
                (AnchorPoint::CenterRight, Point::new(30, 35)),
                (AnchorPoint::BottomLeft, Point::new(10, 50)),
                (AnchorPoint::BottomCenter, Point::new(20, 50)),
                (AnchorPoint::BottomRight, Point::new(30, 50)),
            ],
        );
    }

    #[test]
    fn anchor_point() {
        let rect = Rectangle::new(Point::new(10, 20), Size::new(21, 31));

        for &(anchor_point, expected) in &[
            (AnchorPoint::TopLeft, Point::new(10, 20)),
            (AnchorPoint::TopCenter, Point::new(20, 20)),
            (AnchorPoint::TopRight, Point::new(30, 20)),
            (AnchorPoint::CenterLeft, Point::new(10, 35)),
            (AnchorPoint::Center, Point::new(20, 35)),
            (AnchorPoint::CenterRight, Point::new(30, 35)),
            (AnchorPoint::BottomLeft, Point::new(10, 50)),
            (AnchorPoint::BottomCenter, Point::new(20, 50)),
            (AnchorPoint::BottomRight, Point::new(30, 50)),
        ] {
            assert_eq!(
                rect.anchor_point(anchor_point),
                expected,
                "{:?}",
                anchor_point,
            );

            assert_eq!(
                rect.anchor_x(anchor_point.x()),
                expected.x,
                "{:?}.x()",
                anchor_point
            );

            assert_eq!(
                rect.anchor_y(anchor_point.y()),
                expected.y,
                "{:?}.y()",
                anchor_point
            );
        }
    }

    #[test]
    fn rows_and_columns_zero_sized() {
        let rect = Rectangle::zero();

        assert_eq!(
            rect.rows().next(),
            None,
            "the rows iterator for a zero sized rectangle shouldn't return any items"
        );

        assert_eq!(
            rect.columns().next(),
            None,
            "the columns iterator for a zero sized rectangle shouldn't return any items"
        );
    }
}

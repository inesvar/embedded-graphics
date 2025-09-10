//! A variant of the bresenham algorithm to draw a 1px wide circle.
//! See https://en.wikipedia.org/wiki/Midpoint_circle_algorithm for more details.
//!
//! This module uses x2 zoomed-in coordinates because a circle center can either
//! be on a pixel center or on a pixel intersection.
use crate::{
    geometry::{Point, PointExt},
    pixelcolor::PixelColor,
    primitives::{line::bresenham::MajorMinor, Circle, PrimitiveStyle},
    Pixel,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub(super) struct StyledBresenhamCircleIterator<C> {
    stroke_color: Option<C>,
    circle_iter: BresenhamCircle,
}

impl<C: PixelColor> StyledBresenhamCircleIterator<C> {
    pub(super) fn new(primitive: &Circle, style: &PrimitiveStyle<C>) -> Self {
        // Note: stroke color will be None if stroke width is 0
        let stroke_color = style.effective_stroke_color();

        Self {
            stroke_color,
            circle_iter: BresenhamCircle::new(primitive),
        }
    }
}

impl<C: PixelColor> Iterator for StyledBresenhamCircleIterator<C> {
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let stroke_color = self.stroke_color?;

        self.circle_iter
            .next()
            .map(|point| Pixel(point, stroke_color))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub(super) struct BresenhamCircle {
    /// Current position relative to the circle center
    /// (used to compute error threshold and error step).
    ///
    /// If the accumulated error exceeds the threshold a minor move is made.
    current_position_2x: Point,

    /// Change in position for major and minor steps.
    position_step_2x: MajorMinor<Point>,

    /// Error accumulator.
    error: i32,

    /// Ending position relative to the circle center.
    ending_position_2x: Option<Point>,

    /// Circle center.
    circle_center_2x: Point,
}

impl BresenhamCircle {
    /// Create a new circular bresenham object (the starting position is the rightmost pixel).
    pub(super) fn new(circle: &Circle) -> Self {
        let diameter = circle.diameter;
        let center_offset = if diameter % 2 == 0 {
            1 // the circle center is on a pixel intersection
        } else {
            0 // the circle center is on a pixel center
        };
        let circle_center_2x = circle.center() * 2 + Point::new_equal(center_offset);

        if diameter <= 1 {
            Self {
                current_position_2x: Point::zero(),
                // If diameter is 0 or 1, `position_step_2x` and `error` are unused (less than one pixel is drawn).
                position_step_2x: MajorMinor::new(Point::zero(), Point::zero()),
                error: 0,
                // If the diameter is 0, no pixel is drawn.
                ending_position_2x: (diameter == 0).then(Point::zero),
                circle_center_2x,
            }
        } else {
            Self {
                current_position_2x: Point::new(diameter.saturating_sub(1) as i32, -center_offset),
                position_step_2x: MajorMinor::new(Point::new(0, -2), Point::new(2, 0)),
                error: center_offset * center_offset,
                ending_position_2x: None,
                circle_center_2x,
            }
        }
    }

    /// Compute `position_step_2x` in function of the current octant.
    fn update_position_step(&mut self) {
        let pos = self.current_position_2x;
        if pos == Point::zero() {
            return;
        }
        let y_step = if pos.x > 0 || pos.x == 0 && pos.y > 0 {
            Point::new(0, -2) // to top
        } else {
            Point::new(0, 2) // to bottom
        };

        let x_step = if pos.y < 0 || pos.y == 0 && pos.x > 0 {
            Point::new(-2, 0) // to left
        } else {
            Point::new(2, 0) // to right
        };

        if pos.x.abs() > pos.y.abs() || pos.x == pos.y {
            self.position_step_2x.major = y_step;
            self.position_step_2x.minor = x_step;
        } else {
            self.position_step_2x.major = x_step;
            self.position_step_2x.minor = y_step;
        }
    }

    /// Compute the error increase associated with the major and minor steps.
    fn error_step(&self) -> MajorMinor<i32> {
        // new_error =            dot(pos + step, pos + step)               - radius squared
        // new_error = dot(pos, pos) + 2 * dot(pos, step) + dot(step, step) - radius_squared
        // new_error =   old_error   + 2 * dot(pos, step) + dot(step, step)
        // new_error =   old_error   +     dot(step, 2 * pos + step)
        let delta_error =
            |step: Point| -> i32 { step.dot_product(self.current_position_2x * 2 + step) };

        MajorMinor::new(
            delta_error(self.position_step_2x.major),
            delta_error(self.position_step_2x.minor),
        )
    }
}

impl Iterator for BresenhamCircle {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.current_position_2x;

        if let Some(ending_pos) = self.ending_position_2x {
            if ret == ending_pos {
                return None;
            }
        } else {
            self.ending_position_2x = Some(ret);
        }

        self.update_position_step();
        let error_step = self.error_step();

        self.current_position_2x += self.position_step_2x.major;
        self.error += error_step.major;

        if self.error.abs() > (self.error + error_step.minor).abs() {
            self.current_position_2x += self.position_step_2x.minor;
            self.error += error_step.minor;
        }

        Some((ret + self.circle_center_2x) / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        draw_target::DrawTarget,
        mock_display::MockDisplay,
        pixelcolor::BinaryColor,
        primitives::{styled::StyledDrawable, PrimitiveStyle},
    };

    #[test]
    fn diameter_0_circle_is_correct() {
        let top_lefts = [Point::new(23, 20), Point::new(15, -6), Point::zero()];

        for top_left in top_lefts {
            assert!(BresenhamCircle::new(&Circle::new(top_left, 0)).eq(core::iter::empty()));
        }
    }

    #[test]
    fn diameter_1_circle_is_correct() {
        let top_lefts = [Point::new(-35, -21), Point::new(24, -8), Point::zero()];

        for top_left in top_lefts {
            assert!(BresenhamCircle::new(&Circle::new(top_left, 1)).eq(core::iter::once(top_left)));
        }
    }

    #[test]
    fn starting_circle_pixel_is_correct() {
        let top_left = Point::new(61, 4);

        for diameter in [1, 2, 3, 8, 12, 15, 23, 64] {
            let circle = Circle::new(top_left, diameter);

            assert_eq!(
                BresenhamCircle::new(&circle).next(),
                Some(circle.center() + Point::new(circle.diameter as i32 / 2, 0))
            );
        }
    }

    #[test]
    fn small_bresenham_circle_is_equal_to_circle() {
        let top_left = Point::new(0, 0);
        let stroke_1px_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        for diameter in 0..5 {
            let mut circle_display = MockDisplay::new();
            let circle = Circle::new(top_left, diameter);
            circle
                .draw_styled(&stroke_1px_style, &mut circle_display)
                .unwrap();

            let bresenham_circle = StyledBresenhamCircleIterator::new(&circle, &stroke_1px_style);
            let mut bresenham_circle_display = MockDisplay::new();
            bresenham_circle_display
                .draw_iter(bresenham_circle)
                .unwrap();

            assert_eq!(circle_display, bresenham_circle_display);
        }
    }

    #[test]
    fn circle_bresenham_is_included_in_classic_circle() {
        let top_left = Point::new(0, 0);
        let stroke_1px_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        for diameter in 0..64 {
            let mut circle_display = MockDisplay::new();
            let circle = Circle::new(top_left, diameter);
            circle
                .draw_styled(&stroke_1px_style, &mut circle_display)
                .unwrap();

            let bresenham_circle = BresenhamCircle::new(&circle);

            for pixel in bresenham_circle {
                assert_eq!(circle_display.get_pixel(pixel), Some(BinaryColor::On));
            }
        }
    }
}

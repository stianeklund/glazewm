use serde::{Deserialize, Serialize};

use super::{Direction, LengthValue, Point, RectDelta};

#[derive(Debug, Deserialize, Clone, Serialize, Eq, PartialEq)]
pub struct Rect {
  /// X-coordinate of the left edge of the rectangle.
  pub left: i32,

  /// Y-coordinate of the top edge of the rectangle.
  pub top: i32,

  /// X-coordinate of the right edge of the rectangle.
  pub right: i32,

  /// Y-coordinate of the bottom edge of the rectangle.
  pub bottom: i32,
}

impl Rect {
  /// Creates a new `Rect` instance from the coordinates of its left, top,
  /// right, and bottom edges.
  #[must_use]
  pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }

  /// Creates a new `Rect` instance from its X/Y coordinates and size.
  #[must_use]
  pub fn from_xy(x: i32, y: i32, width: i32, height: i32) -> Self {
    Self {
      left: x,
      top: y,
      right: x + width,
      bottom: y + height,
    }
  }

  #[must_use]
  pub fn x(&self) -> i32 {
    self.left
  }

  #[must_use]
  pub fn y(&self) -> i32 {
    self.top
  }

  #[must_use]
  pub fn width(&self) -> i32 {
    self.right - self.left
  }

  #[must_use]
  pub fn height(&self) -> i32 {
    self.bottom - self.top
  }

  #[must_use]
  pub fn translate_to_coordinates(&self, x: i32, y: i32) -> Self {
    Self::from_xy(x, y, self.width(), self.height())
  }

  #[must_use]
  pub fn translate_to_center(&self, outer_rect: &Rect) -> Self {
    Self::translate_to_coordinates(
      self,
      outer_rect.left + (outer_rect.width() / 2) - (self.width() / 2),
      outer_rect.top + (outer_rect.height() / 2) - (self.height() / 2),
    )
  }

  #[must_use]
  pub fn translate_in_direction(
    &self,
    direction: &Direction,
    distance: i32,
  ) -> Rect {
    let (delta_x, delta_y) = match direction {
      Direction::Up => (0, -distance),
      Direction::Down => (0, distance),
      Direction::Left => (-distance, 0),
      Direction::Right => (distance, 0),
    };

    Self::from_xy(
      self.x() + delta_x,
      self.y() + delta_y,
      self.width(),
      self.height(),
    )
  }

  /// Returns a new `Rect` that is clamped within the bounds of the given
  /// outer rectangle. Attempts to preserve the width and height of the
  /// original rectangle.
  #[must_use]
  pub fn clamp(&self, outer_rect: &Rect) -> Self {
    Self::from_xy(
      self.left.max(outer_rect.left),
      self.top.max(outer_rect.top),
      self.width().min(outer_rect.width()),
      self.height().min(outer_rect.height()),
    )
  }

  /// Returns a new `Rect` that is strictly constrained within the bounds
  /// of the given outer rectangle. If the window would extend beyond the
  /// bounds, it is repositioned to fit entirely within the outer
  /// rectangle. Preserves the original width and height when possible.
  #[must_use]
  pub fn clamp_within_bounds(&self, outer_rect: &Rect) -> Self {
    let clamped_width = self.width().min(outer_rect.width());
    let clamped_height = self.height().min(outer_rect.height());

    let mut x = self.x();
    let mut y = self.y();

    // Ensure window doesn't overflow right edge
    if x + clamped_width > outer_rect.right {
      x = outer_rect.right - clamped_width;
    }

    // Ensure window doesn't overflow bottom edge
    if y + clamped_height > outer_rect.bottom {
      y = outer_rect.bottom - clamped_height;
    }

    // Ensure window doesn't underflow left/top edges
    x = x.max(outer_rect.left);
    y = y.max(outer_rect.top);

    Self::from_xy(x, y, clamped_width, clamped_height)
  }

  #[must_use]
  pub fn clamp_size(&self, width: i32, height: i32) -> Self {
    Self::from_xy(
      self.x(),
      self.y(),
      self.width().min(width),
      self.height().min(height),
    )
  }

  #[must_use]
  pub fn center_point(&self) -> Point {
    Point {
      x: self.left + (self.width() / 2),
      y: self.top + (self.height() / 2),
    }
  }

  /// Gets the delta between this rect and another rect.
  #[must_use]
  pub fn delta(&self, other: &Rect) -> RectDelta {
    RectDelta {
      left: LengthValue::from_px(other.left - self.left),
      top: LengthValue::from_px(other.top - self.top),
      right: LengthValue::from_px(self.right - other.right),
      bottom: LengthValue::from_px(self.bottom - other.bottom),
    }
  }

  #[must_use]
  pub fn apply_delta(
    &self,
    delta: &RectDelta,
    scale_factor: Option<f32>,
  ) -> Self {
    Self::from_ltrb(
      self.left - delta.left.to_px(self.width(), scale_factor),
      self.top - delta.top.to_px(self.height(), scale_factor),
      self.right + delta.right.to_px(self.width(), scale_factor),
      self.bottom + delta.bottom.to_px(self.height(), scale_factor),
    )
  }

  #[must_use]
  pub fn apply_inverse_delta(
    &self,
    delta: &RectDelta,
    scale_factor: Option<f32>,
  ) -> Self {
    Self::from_ltrb(
      self.left + delta.left.to_px(self.width(), scale_factor),
      self.top + delta.top.to_px(self.height(), scale_factor),
      self.right - delta.right.to_px(self.width(), scale_factor),
      self.bottom - delta.bottom.to_px(self.height(), scale_factor),
    )
  }

  // Gets whether the x-coordinate overlaps with the x-coordinate of the
  // other rect.
  #[must_use]
  pub fn has_overlap_x(&self, other: &Rect) -> bool {
    !(self.x() + self.width() <= other.x()
      || other.x() + other.width() <= self.x())
  }

  // Gets whether the y-coordinate overlaps with the y-coordinate of the
  // other rect.
  #[must_use]
  pub fn has_overlap_y(&self, other: &Rect) -> bool {
    !(self.y() + self.height() <= other.y()
      || other.y() + other.height() <= self.y())
  }

  #[must_use]
  pub fn contains_point(&self, point: &Point) -> bool {
    let is_in_x = point.x >= self.left && point.x <= self.right;
    let is_in_y = point.y >= self.top && point.y <= self.bottom;
    is_in_x && is_in_y
  }

  #[must_use]
  pub fn distance_to_point(&self, point: &Point) -> f32 {
    let dx = (self.x() - point.x)
      .abs()
      .max((self.x() + self.width() - point.x).abs());

    let dy = (self.y() - point.y)
      .abs()
      .max((self.y() + self.height() - point.y).abs());

    #[allow(clippy::cast_precision_loss)]
    ((dx * dx + dy * dy) as f32).sqrt()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_clamp_within_bounds_no_overflow() {
    let window_rect = Rect::from_xy(100, 100, 200, 150);
    let monitor_rect = Rect::from_xy(0, 0, 1920, 1080);

    let result = window_rect.clamp_within_bounds(&monitor_rect);

    // Window should remain unchanged as it fits within bounds
    assert_eq!(result, window_rect);
  }

  #[test]
  fn test_clamp_within_bounds_right_overflow() {
    let window_rect = Rect::from_xy(1800, 100, 200, 150);
    let monitor_rect = Rect::from_xy(0, 0, 1920, 1080);

    let result = window_rect.clamp_within_bounds(&monitor_rect);

    // Window should be repositioned to fit entirely within monitor
    assert_eq!(result, Rect::from_xy(1720, 100, 200, 150));
  }

  #[test]
  fn test_clamp_within_bounds_bottom_overflow() {
    let window_rect = Rect::from_xy(100, 1000, 200, 150);
    let monitor_rect = Rect::from_xy(0, 0, 1920, 1080);

    let result = window_rect.clamp_within_bounds(&monitor_rect);

    // Window should be repositioned to fit entirely within monitor
    assert_eq!(result, Rect::from_xy(100, 930, 200, 150));
  }

  #[test]
  fn test_clamp_within_bounds_mixed_resolution_scenario() {
    // Simulate Task Manager spilling from 4K monitor to smaller monitor
    // 4K monitor: 0,0 to 3840,2160
    // 1920x1200 monitor: 3840,0 to 5760,1200
    let window_rect = Rect::from_xy(3700, 100, 300, 400); // Window near edge of 4K monitor
    let small_monitor_rect = Rect::from_xy(3840, 0, 1920, 1200);

    let result = window_rect.clamp_within_bounds(&small_monitor_rect);

    // Should be fully contained within the small monitor
    assert_eq!(result, Rect::from_xy(3840, 100, 300, 400));
    assert!(result.right <= small_monitor_rect.right);
    assert!(result.bottom <= small_monitor_rect.bottom);
  }

  #[test]
  fn test_clamp_within_bounds_oversized_window() {
    let window_rect = Rect::from_xy(100, 100, 2000, 1500);
    let monitor_rect = Rect::from_xy(0, 0, 1920, 1080);

    let result = window_rect.clamp_within_bounds(&monitor_rect);

    // Window should be resized and positioned at origin
    assert_eq!(result, Rect::from_xy(0, 0, 1920, 1080));
  }

  #[test]
  fn test_clamp_within_bounds_negative_coordinates() {
    let window_rect = Rect::from_xy(-100, -50, 200, 150);
    let monitor_rect = Rect::from_xy(0, 0, 1920, 1080);

    let result = window_rect.clamp_within_bounds(&monitor_rect);

    // Window should be repositioned to monitor origin
    assert_eq!(result, Rect::from_xy(0, 0, 200, 150));
  }

  #[test]
  fn test_clamp_within_bounds_multi_monitor_boundary() {
    // Test window at boundary between monitors in mixed resolution setup
    let window_rect = Rect::from_xy(3700, 500, 400, 300); // Spans across monitors
    let right_monitor_rect = Rect::from_xy(3840, 0, 1920, 1200);

    let result = window_rect.clamp_within_bounds(&right_monitor_rect);

    // Should be fully within the right monitor
    assert_eq!(result, Rect::from_xy(3840, 500, 400, 300));
    assert!(result.left >= right_monitor_rect.left);
    assert!(result.right <= right_monitor_rect.right);
  }
}

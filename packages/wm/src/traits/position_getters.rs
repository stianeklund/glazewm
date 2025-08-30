use ambassador::delegatable_trait;
use wm_common::Rect;

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized. This is used by `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let parent_rect = parent.to_rect()?;

        // Parent rect logging removed for clarity

        let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
        let inner_gap = match parent.tiling_direction() {
          TilingDirection::Vertical => vertical_gap,
          TilingDirection::Horizontal => horizontal_gap,
        };

        #[allow(
          clippy::cast_precision_loss,
          clippy::cast_possible_truncation,
          clippy::cast_possible_wrap
        )]
        let (width, height) = match parent.tiling_direction() {
          TilingDirection::Vertical => {
            let sibling_count = self.tiling_siblings().count() as i32;
            let available_height =
              parent_rect.height() - inner_gap * sibling_count;

            // Provisional height based on tiling_size with rounding.
            let mut height = (available_height as f32 * self.tiling_size())
              .round() as i32;

            // Vertical tiling logging removed for clarity

            // If this is the last tiling sibling in a vertical split,
            // fill the remaining space exactly to avoid rounding gaps.
            let is_last = self
              .next_siblings()
              .filter_map(|s| s.as_tiling_container().ok())
              .next()
              .is_none();

            let (_x, y) = {
              let mut prev_siblings = self
                .prev_siblings()
                .filter_map(|sibling| sibling.as_tiling_container().ok());

              match prev_siblings.next() {
                None => (parent_rect.x(), parent_rect.y()),
                Some(sibling) => {
                  let sibling_rect = sibling.to_rect()?;

                  (
                    parent_rect.x(),
                    sibling_rect.y() + sibling_rect.height() + inner_gap,
                  )
                }
              }
            };

            if is_last {
              // Height = bottom of parent - our y
              height = parent_rect.bottom - y;
            }

            (parent_rect.width(), height)
          }
          TilingDirection::Horizontal => {
            let sibling_count = self.tiling_siblings().count() as i32;
            let _total_tiling_containers = sibling_count + 1;
            let available_width =
              parent_rect.width() - inner_gap * sibling_count;

            // Provisional width based on tiling_size with rounding.
            let mut width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            // If this is the last tiling sibling in a horizontal split,
            // fill the remaining space exactly to avoid rounding gaps.
            let is_last = self
              .next_siblings()
              .filter_map(|s| s.as_tiling_container().ok())
              .next()
              .is_none();

            let (x, _y) = {
              let mut prev_siblings = self
                .prev_siblings()
                .filter_map(|sibling| sibling.as_tiling_container().ok());

              match prev_siblings.next() {
                None => (parent_rect.x(), parent_rect.y()),
                Some(sibling) => {
                  let sibling_rect = sibling.to_rect()?;

                  (
                    sibling_rect.x() + sibling_rect.width() + inner_gap,
                    parent_rect.y(),
                  )
                }
              }
            };

            let _original_width = width;
            if is_last {
              // Width = right of parent - our x
              width = parent_rect.right - x;
            }

            (width, parent_rect.height())
          }
        };

        // Recompute position to return with the final width/height.
        let (x, y) = {
          let mut prev_siblings = self
            .prev_siblings()
            .filter_map(|sibling| sibling.as_tiling_container().ok());

          match prev_siblings.next() {
            None => (parent_rect.x(), parent_rect.y()),
            Some(sibling) => {
              let sibling_rect = sibling.to_rect()?;

              let final_x = match parent.tiling_direction() {
                TilingDirection::Vertical => parent_rect.x(),
                TilingDirection::Horizontal => {
                  sibling_rect.x() + sibling_rect.width() + inner_gap
                }
              };
              let final_y = match parent.tiling_direction() {
                TilingDirection::Vertical => {
                  sibling_rect.y() + sibling_rect.height() + inner_gap
                }
                TilingDirection::Horizontal => parent_rect.y(),
              };

              (final_x, final_y)
            }
          }
        };

        let final_rect = Rect::from_xy(x, y, width, height);

        Ok(final_rect)
      }
    }
  };
}

use image::{RgbImage, imageops};

/// Create a side-by-side composite image showing baseline, diff, and current.
///
/// The resulting image is 3x the width of the input images and the same height.
/// Layout: `[baseline | diff | current]`
///
/// # Arguments
///
/// * `baseline` - The reference/expected image (left panel)
/// * `diff` - The diff visualization image (center panel)
/// * `current` - The newly captured image (right panel)
///
/// # Panics
///
/// All three images should have the same dimensions for meaningful output.
pub fn create_composite(baseline: &RgbImage, diff: &RgbImage, current: &RgbImage) -> RgbImage {
    let width = baseline.width();
    let height = baseline.height();
    let mut composite = RgbImage::new(width * 3, height);

    imageops::overlay(&mut composite, baseline, 0, 0);
    imageops::overlay(&mut composite, diff, width as i64, 0);
    imageops::overlay(&mut composite, current, (width * 2) as i64, 0);

    composite
}

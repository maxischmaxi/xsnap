use image::RgbImage;
use image::imageops;

use crate::error::XsnapError;

/// Result of comparing two images.
#[derive(Debug)]
pub enum CompareResult {
    /// Images are similar enough (within the pixel threshold).
    Pass,
    /// Images differ beyond the threshold.
    Fail {
        /// Similarity score from the comparison algorithm.
        /// 1.0 means identical, lower values indicate more differences.
        score: f64,
        /// Optional diff visualization image.
        diff_image: Option<RgbImage>,
    },
}

/// Compare two RGB images and determine if they pass a pixel-difference threshold.
///
/// Uses hybrid structural/color comparison (MSSIM on Y channel, RMS on U/V channels).
/// The score is approximately 1.0 for identical images and decreases toward 0.0 for
/// completely different images.
///
/// When dimensions differ, the images are cropped to their overlapping area before
/// comparison. Width mismatches produce a warning (may indicate a UI bug). Height
/// mismatches are expected (e.g. mobile views with varying content length) and are
/// handled silently.
///
/// # Arguments
///
/// * `baseline` - The reference/expected image
/// * `current` - The newly captured image
/// * `threshold_pixels` - Maximum number of differing pixels allowed before failing
/// * `threshold_percent` - Maximum percentage (0.0–100.0) of differing pixels allowed
///
/// The comparison **passes** when *either* threshold is satisfied.
///
/// # Returns
///
/// A tuple of `(CompareResult, Vec<String>)` where the second element contains
/// any warnings generated during comparison (e.g. width mismatch).
///
/// # Errors
///
/// Returns `XsnapError::DiffFailed` if the comparison algorithm fails.
pub fn compare_images(
    baseline: &RgbImage,
    current: &RgbImage,
    threshold_pixels: u32,
    threshold_percent: f64,
) -> Result<(CompareResult, Vec<String>), XsnapError> {
    let mut warnings = Vec::new();

    let (b_w, b_h) = baseline.dimensions();
    let (c_w, c_h) = current.dimensions();

    // Check for width mismatch (may indicate a UI bug).
    if b_w != c_w {
        warnings.push(format!(
            "Width mismatch: baseline {}px vs current {}px",
            b_w, c_w
        ));
    }

    // Crop both images to the overlapping area for comparison.
    let (baseline_cmp, current_cmp) = if (b_w, b_h) != (c_w, c_h) {
        let min_w = b_w.min(c_w);
        let min_h = b_h.min(c_h);
        (
            imageops::crop_imm(baseline, 0, 0, min_w, min_h).to_image(),
            imageops::crop_imm(current, 0, 0, min_w, min_h).to_image(),
        )
    } else {
        (baseline.clone(), current.clone())
    };

    let result = image_compare::rgb_hybrid_compare(&baseline_cmp, &current_cmp).map_err(|e| {
        XsnapError::DiffFailed {
            message: format!("Comparison failed: {}", e),
        }
    })?;

    let total_pixels = baseline_cmp.width() * baseline_cmp.height();
    let diff_pixels = ((1.0 - result.score) * total_pixels as f64) as u32;
    let diff_pct = if total_pixels > 0 {
        (diff_pixels as f64 / total_pixels as f64) * 100.0
    } else {
        0.0
    };

    if diff_pixels <= threshold_pixels || diff_pct <= threshold_percent {
        Ok((CompareResult::Pass, warnings))
    } else {
        // Convert the similarity map to a viewable RGB image
        // TODO: Apply user-configured diff_pixel_color instead of using default color map
        let diff_image = result.image.to_color_map().to_rgb8();
        Ok((
            CompareResult::Fail {
                score: result.score,
                diff_image: Some(diff_image),
            },
            warnings,
        ))
    }
}

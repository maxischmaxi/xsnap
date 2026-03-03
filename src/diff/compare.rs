use image::RgbImage;

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
/// # Arguments
///
/// * `baseline` - The reference/expected image
/// * `current` - The newly captured image
/// * `threshold_pixels` - Maximum number of differing pixels allowed before failing
///
/// # Errors
///
/// Returns `XsnapError::DiffFailed` if dimensions differ or the comparison algorithm fails.
pub fn compare_images(
    baseline: &RgbImage,
    current: &RgbImage,
    threshold_pixels: u32,
) -> Result<CompareResult, XsnapError> {
    if baseline.dimensions() != current.dimensions() {
        return Err(XsnapError::DiffFailed {
            message: format!(
                "Dimension mismatch: baseline {:?} vs current {:?}",
                baseline.dimensions(),
                current.dimensions()
            ),
        });
    }

    let result =
        image_compare::rgb_hybrid_compare(baseline, current).map_err(|e| {
            XsnapError::DiffFailed {
                message: format!("Comparison failed: {}", e),
            }
        })?;

    let total_pixels = baseline.width() * baseline.height();
    let diff_pixels = ((1.0 - result.score) * total_pixels as f64) as u32;

    if diff_pixels <= threshold_pixels {
        Ok(CompareResult::Pass)
    } else {
        // Convert the similarity map to a viewable RGB image
        let diff_image = result.image.to_color_map().to_rgb8();
        Ok(CompareResult::Fail {
            score: result.score,
            diff_image: Some(diff_image),
        })
    }
}

use image::{Rgb, RgbImage};
use xsnap::diff::compare::{CompareResult, compare_images};
use xsnap::diff::composite::create_composite;

fn create_solid_image(width: u32, height: u32, color: [u8; 3]) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgb(color);
    }
    img
}

#[test]
fn test_identical_images_pass() {
    let img = create_solid_image(100, 100, [128, 128, 128]);
    let (result, warnings) = compare_images(&img, &img, 0, 0.0).expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Pass),
        "identical images should pass with threshold 0"
    );
    assert!(warnings.is_empty());
}

#[test]
fn test_different_images_fail() {
    let baseline = create_solid_image(100, 100, [0, 0, 0]);
    let current = create_solid_image(100, 100, [255, 255, 255]);
    let (result, _) =
        compare_images(&baseline, &current, 0, 0.0).expect("comparison should succeed");
    match result {
        CompareResult::Fail { score, diff_image } => {
            assert!(
                score < 1.0,
                "score should be less than 1.0 for different images"
            );
            assert!(diff_image.is_some(), "diff image should be present");
        }
        CompareResult::Pass => panic!("completely different images should not pass"),
    }
}

#[test]
fn test_threshold_allows_small_diff() {
    let mut baseline = create_solid_image(100, 100, [128, 128, 128]);
    let current = baseline.clone();

    // Modify 5 pixels in the baseline to create a small difference
    for x in 0..5 {
        baseline.put_pixel(x, 0, Rgb([0, 0, 0]));
    }

    // First verify that a threshold of 0 detects the difference
    let (strict_result, _) =
        compare_images(&baseline, &current, 0, 0.0).expect("comparison should succeed");
    let diff_pixel_count = match &strict_result {
        CompareResult::Fail { score, .. } => {
            let total = 100u32 * 100;
            ((1.0 - score) * total as f64) as u32
        }
        CompareResult::Pass => 0,
    };

    // Now verify that a sufficiently high threshold lets it pass.
    // MSSIM works on windows so the affected pixel count may be larger
    // than the literal number of changed pixels.
    let generous_threshold = diff_pixel_count + 50;
    let (result, _) = compare_images(&baseline, &current, generous_threshold, 0.0)
        .expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Pass),
        "small diff within generous threshold should pass, diff_pixel_count was {}",
        diff_pixel_count
    );
}

#[test]
fn test_dimension_mismatch_width_warns() {
    let narrow = create_solid_image(50, 100, [128, 128, 128]);
    let wide = create_solid_image(100, 100, [128, 128, 128]);
    let (result, warnings) =
        compare_images(&narrow, &wide, 0, 0.0).expect("width mismatch should not error");
    // Should still produce a comparison result (not an error).
    assert!(
        matches!(result, CompareResult::Pass | CompareResult::Fail { .. }),
        "width mismatch should produce a comparison result"
    );
    assert!(
        warnings.iter().any(|w| w.contains("Width mismatch")),
        "should warn about width mismatch, got: {:?}",
        warnings
    );
}

#[test]
fn test_dimension_mismatch_height_no_warning() {
    let short = create_solid_image(100, 50, [128, 128, 128]);
    let tall = create_solid_image(100, 100, [128, 128, 128]);
    let (_, warnings) =
        compare_images(&short, &tall, 0, 0.0).expect("height mismatch should not error");
    assert!(
        warnings.is_empty(),
        "height-only mismatch should not produce warnings, got: {:?}",
        warnings
    );
}

#[test]
fn test_threshold_percent_allows_small_diff() {
    let baseline = create_solid_image(100, 100, [128, 128, 128]);
    let current = create_solid_image(100, 100, [255, 255, 255]);

    // With 0% tolerance both thresholds reject
    let (strict, _) =
        compare_images(&baseline, &current, 0, 0.0).expect("comparison should succeed");
    assert!(
        matches!(strict, CompareResult::Fail { .. }),
        "completely different images should fail with 0% threshold"
    );

    // With 100% tolerance the percent threshold lets it pass
    let (lenient, _) =
        compare_images(&baseline, &current, 0, 100.0).expect("comparison should succeed");
    assert!(
        matches!(lenient, CompareResult::Pass),
        "100% threshold should let anything pass"
    );
}

#[test]
fn test_threshold_percent_either_threshold_passes() {
    let mut baseline = create_solid_image(100, 100, [128, 128, 128]);
    let current = baseline.clone();

    // Modify a few pixels
    for x in 0..5 {
        baseline.put_pixel(x, 0, Rgb([0, 0, 0]));
    }

    // Pixel threshold 0, percent threshold 0 → should fail
    let (result, _) =
        compare_images(&baseline, &current, 0, 0.0).expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Fail { .. }),
        "both thresholds at 0 should fail"
    );

    // Pixel threshold 0, percent threshold generous → should pass via percent
    let (result, _) =
        compare_images(&baseline, &current, 0, 5.0).expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Pass),
        "generous percent threshold should pass even with pixel threshold 0"
    );
}

#[test]
fn test_create_composite_dimensions() {
    let width = 80;
    let height = 60;
    let baseline = create_solid_image(width, height, [255, 0, 0]);
    let diff = create_solid_image(width, height, [0, 255, 0]);
    let current = create_solid_image(width, height, [0, 0, 255]);

    let composite = create_composite(&baseline, &diff, &current);

    assert_eq!(
        composite.width(),
        width * 3,
        "composite width should be 3x input width"
    );
    assert_eq!(
        composite.height(),
        height,
        "composite height should match input height"
    );
}

#[test]
fn test_create_composite_contains_all_images() {
    let width = 80;
    let height = 60;
    let red = [255, 0, 0];
    let green = [0, 255, 0];
    let blue = [0, 0, 255];

    let baseline = create_solid_image(width, height, red);
    let diff = create_solid_image(width, height, green);
    let current = create_solid_image(width, height, blue);

    let composite = create_composite(&baseline, &diff, &current);

    // Check a pixel in the baseline section (left third)
    let p = composite.get_pixel(10, 10);
    assert_eq!(p.0, red, "left section should contain baseline (red)");

    // Check a pixel in the diff section (middle third)
    let p = composite.get_pixel(width + 10, 10);
    assert_eq!(p.0, green, "middle section should contain diff (green)");

    // Check a pixel in the current section (right third)
    let p = composite.get_pixel(width * 2 + 10, 10);
    assert_eq!(p.0, blue, "right section should contain current (blue)");
}

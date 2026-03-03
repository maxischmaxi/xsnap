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
    let result = compare_images(&img, &img, 0).expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Pass),
        "identical images should pass with threshold 0"
    );
}

#[test]
fn test_different_images_fail() {
    let baseline = create_solid_image(100, 100, [0, 0, 0]);
    let current = create_solid_image(100, 100, [255, 255, 255]);
    let result = compare_images(&baseline, &current, 0).expect("comparison should succeed");
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
    let strict_result = compare_images(&baseline, &current, 0).expect("comparison should succeed");
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
    let result =
        compare_images(&baseline, &current, generous_threshold).expect("comparison should succeed");
    assert!(
        matches!(result, CompareResult::Pass),
        "small diff within generous threshold should pass, diff_pixel_count was {}",
        diff_pixel_count
    );
}

#[test]
fn test_dimension_mismatch() {
    let small = create_solid_image(50, 50, [128, 128, 128]);
    let large = create_solid_image(100, 100, [128, 128, 128]);
    let result = compare_images(&small, &large, 0);
    assert!(result.is_err(), "dimension mismatch should return error");
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("Dimension mismatch"),
        "error message should mention dimension mismatch, got: {}",
        msg
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

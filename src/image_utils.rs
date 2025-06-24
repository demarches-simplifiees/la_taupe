use std::env::{current_dir, var};

use image::{DynamicImage, ImageBuffer, Rgb};
use imageproc::{
    edges::canny,
    geometric_transformations::{warp_into, Interpolation, Projection},
    hough::{detect_lines, draw_polar_lines, LineDetectionOptions, PolarLine},
    map::map_pixels,
};
use linfa::prelude::Predict;
use linfa::{traits::Fit, Dataset};
use linfa_clustering::KMeans;
use ndarray::Array2;

const WHITE: Rgb<u8> = Rgb::<u8>([255, 255, 255]);
const GREEN: Rgb<u8> = Rgb::<u8>([0, 255, 0]);
const BLACK: Rgb<u8> = Rgb::<u8>([0, 0, 0]);

pub fn clean_image(image: &DynamicImage, name: &str) -> DynamicImage {
    let angle = angle(image, name);

    let image = rotate(image, -angle);

    save_image_in_debug(&image, name, "cleaned");

    image
}

fn angle(image: &DynamicImage, name: &str) -> f32 {
    let gray_image = image.clone().into_luma8();
    let edges = canny(&gray_image, 50.0, 100.0);

    let options = LineDetectionOptions {
        vote_threshold: 400,
        suppression_radius: 0,
    };
    let lines: Vec<PolarLine> = detect_lines(&edges, options);

    if var("DEBUG_IMAGE").is_ok() {
        draw_lines(&edges.into(), &lines, name);
    }

    let angles: Vec<f64> = lines
        .iter()
        .map(|line| line.angle_in_degrees as f64)
        .collect();

    if angles.is_empty() {
        log::trace!("No angles detected, returning 0.0");
        return 0.0;
    }

    let angle = main_cluster_mean_angle(angles);
    log::trace!("angle detected: {}", angle);

    (angle - 90.0).to_radians() as f32
}

fn rotate(image: &DynamicImage, theta: f32) -> DynamicImage {
    let image = image.to_rgb8();

    let (width, height) = (image.width() as f32, image.height() as f32);
    let (new_width, new_height) = (
        (width * theta.cos().abs() + height * theta.sin().abs()),
        (height * theta.cos().abs() + width * theta.sin().abs()),
    );

    let (cx, cy) = (width / 2f32, height / 2f32);
    let (new_cx, new_cy) = ((new_width / 2f32), (new_height / 2f32));

    let mut new_image = ImageBuffer::from_pixel(new_width as u32, new_height as u32, WHITE);
    let projection = Projection::translate(new_cx, new_cy)
        * Projection::rotate(theta)
        * Projection::translate(-cx, -cy);

    warp_into(
        &image,
        &projection,
        Interpolation::Bilinear,
        WHITE,
        &mut new_image,
    );

    new_image.into()
}

fn main_cluster_mean_angle(angles: Vec<f64>) -> f64 {
    let n = angles.len();
    let array: Array2<f64> = Array2::from_shape_vec((n, 1), angles.to_vec()).unwrap();
    let dataset = Dataset::from(array.clone());

    let model = KMeans::params(2)
        .max_n_iterations(100)
        .fit(&dataset)
        .expect("Failed to fit KMeans model");

    let labels = model.predict(array.clone());

    let mut cluster_0 = Vec::new();
    let mut cluster_1 = Vec::new();

    for (i, &label) in labels.targets.iter().enumerate() {
        if label == 0 {
            cluster_0.push(angles[i]);
        } else {
            cluster_1.push(angles[i]);
        }
    }

    let values = if cluster_0.len() >= cluster_1.len() {
        cluster_0
    } else {
        cluster_1
    };

    values.iter().sum::<f64>() / values.len() as f64
}

fn draw_lines(edges: &DynamicImage, lines: &[PolarLine], name: &str) {
    let color_edges = map_pixels(edges, |_x, _y, p| if p[0] > 0 { WHITE } else { BLACK });
    let lines_image = draw_polar_lines(&color_edges, lines, GREEN);
    save_image_in_debug(&lines_image.into(), name, "lines");
}

fn save_image_in_debug(image: &DynamicImage, name: &str, suffix: &str) {
    if var("DEBUG_IMAGE").is_err() {
        return;
    }

    let debug_dir = current_dir()
        .expect("Failed to get current directory")
        .join("la_taupe_debug");

    std::fs::create_dir_all(&debug_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create debug directory: {}, {}",
            debug_dir.display(),
            e
        )
    });

    image
        .save(debug_dir.join(format!("{}_{}.png", name, suffix)))
        .unwrap_or_else(|e| panic!("Failed to save image: {}, {}", name, e));
}

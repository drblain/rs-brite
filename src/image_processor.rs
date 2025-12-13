use opencv::prelude::*;
use anyhow::{anyhow, Result};
use brightness::blocking::Brightness;
use opencv::{
    core::{Scalar, Mat, CV_32F},
    videoio::{VideoCapture, CAP_ANY}
};
use std::sync::{Arc, LazyLock};

pub struct Webcam(VideoCapture);

impl Webcam {
    pub fn new(index: i32) -> Result<Self> {
        let camera = VideoCapture::new(0, CAP_ANY)?;

        VideoCapture::is_opened(&camera)?
            .then_some(Self(camera))
            .ok_or_else(|| anyhow!("Failed to open camera at index {}", index))
    }

    pub fn read_frame(&mut self) -> Result<Mat> {
        let mut frame = Mat::default();
        self.0.read(&mut frame)?;

        if frame.empty() {
            return Err(anyhow!("Failed to capture frame"));
        }

        Ok(frame)
    }
}

const FLOAT_COMPRESSED_TO_LINEAR: f64 = 2.2;
const FLOAT_MAX_COLOR: f64 = 255.0;
const FLOAT_MAX_PERCENT: f64 = 100.0;
const SCREEN_BRIGHTNESS_DEFAULT: u8 = 50;
const SCREEN_BRIGHTNESS_MIN: u32 = 1;
const SCREEN_REFLECTION_FACTOR: f64 = 0.2;
static COEFFS_BGR_LUMA: LazyLock<Arc<[f64]>> = LazyLock::new(|| {
    Arc::from(vec![0.0722, 0.7152, 0.2126])
});

pub fn compute_raw_luma(raw_frame: &Mat) -> Result<f64> {
    let mut float_img = Mat::default();
    raw_frame.convert_to(&mut float_img, CV_32F, 1.0 / FLOAT_MAX_COLOR, 0.0)?;

    let mut linear_img = Mat::default();
    opencv::core::pow(&float_img, FLOAT_COMPRESSED_TO_LINEAR, &mut linear_img)?;

    let mut luma_img = Mat::default();
    let coeff_mat = Mat::from_slice(&*COEFFS_BGR_LUMA)?;
    opencv::core::transform(&linear_img, &mut luma_img, &coeff_mat)?;

    let raw_luma: Scalar = opencv::core::mean(&luma_img, &opencv::core::no_array())?;

    Ok(raw_luma[0] * FLOAT_MAX_COLOR)
}

pub fn adjusted_luma(raw_luma: f64) -> Result<f64> {
    let current_brightness = get_screen_brightness()
        .unwrap_or(SCREEN_BRIGHTNESS_DEFAULT);

    // could put logic to grab the current screen content
    // and multiply the screen brightness by its luma

    let screen_contribution = (current_brightness as f64 / FLOAT_MAX_PERCENT * FLOAT_MAX_COLOR) * SCREEN_REFLECTION_FACTOR;

    let adjusted_luma = raw_luma - screen_contribution;

    println!("[Processor] Raw Luma: {:.2} - Screen Contribution: {:.2} = Adjusted Luma: {:.2}",
        raw_luma, screen_contribution, adjusted_luma);

    Ok(adjusted_luma.max(0.0) as f64)
}

pub fn compute_luma(image: &Mat) -> Result<u8> {
    let raw_luma = compute_raw_luma(image)?;
    let adj_luma = adjusted_luma(raw_luma)?;

    Ok(adj_luma as u8)
}

pub fn get_screen_brightness() -> Result<u8> {
    for device in brightness::blocking::brightness_devices() {
        if let Ok(dev) = device {
            return Ok(dev.get()? as u8);
        }
    }
    Err(anyhow!("No primary brightness device found"))
}

pub fn set_screen_brightness(percent: u32) -> Result<()> {
    let percent_actual: u32 = percent.max(SCREEN_BRIGHTNESS_MIN) as u32;
    // set the brightness for the first (primary) brightness device
    for device in brightness::blocking::brightness_devices() {
        if let Ok(dev) = device {
            dev.set(percent_actual)?;
            return Ok(());
        }
    }

    // no device was found iterating on ALL the devices
    Err(anyhow!("No primary brightness device found"))
}

pub fn auto_brightness(camera: &mut Webcam) -> Result<()> {
    let frame_image = camera.read_frame()?;

    let avg_luma = compute_luma(&frame_image)?;

    let percent_brightness = (avg_luma as f64 / FLOAT_MAX_COLOR * FLOAT_MAX_PERCENT) as u32;
    println!("[Processor] Luma: {}/{} -> Setting: {}%", avg_luma, FLOAT_MAX_COLOR, percent_brightness);
    set_screen_brightness(percent_brightness)?;

    Ok(())
}
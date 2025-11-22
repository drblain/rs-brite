use anyhow::{anyhow, Result};
use brightness::blocking::Brightness;
use image::{ImageBuffer, Pixel, Rgb};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera
};
use rayon::prelude::*;

const SIZE_CAMERA_WARMUP_FRAMES: usize = 5;
const SIZE_RGB_CHUNK: usize = 3;
const FLOAT_MAX_COLOR: f32 = 255.0;
const FLOAT_MAX_PERCENT: f32 = 100.0;
const SCREEN_BRIGHTNESS_DEFAULT: u8 = 50;
const SCREEN_REFLECTION_FACTOR: f32 = 0.2;
type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub fn setup_camera() -> Result<Camera> {
    // Get the first camera
    let index = CameraIndex::Index(0);
    let fmt_requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let camera = Camera::new(index, fmt_requested)?;

    Ok(camera)
}

fn capture_frame(camera: &mut Camera) -> Result<RgbImage> {
    camera.open_stream()?;

    // dump warmup frames in the trash
    for _ in 0..SIZE_CAMERA_WARMUP_FRAMES {
        let _ = camera.frame()?;
    }

    // capture a single usable frame and immediately close the camera
    let frame_buffer = camera.frame()?;
    camera.stop_stream()?;

    // convert image from stream to RGB for use
    let image: RgbImage = frame_buffer.decode_image::<RgbFormat>()?;

    Ok(image)
}

fn compute_raw_luma(image: &RgbImage) -> Result<f32> {
    let raw_buffer = image.as_raw();

    let pixel_count = (raw_buffer.len() / SIZE_RGB_CHUNK) as u64;

    if pixel_count == 0 {
        return Err(anyhow!("No pixels found in image"));
    }

    let total_luma: u64 = raw_buffer
        .par_chunks(SIZE_RGB_CHUNK)
        .map(|chunk| {
            let p = image::Rgb::from_slice(chunk);
            p.to_luma()[0] as u64
        })
        .sum();

    Ok((total_luma / pixel_count) as f32)
}

fn adjusted_luma(raw_luma: f32) -> Result<f32> {
    let current_brightness = get_screen_brightness()
        .unwrap_or(SCREEN_BRIGHTNESS_DEFAULT);

    // could put logic to grab the current screen content
    // and multiply the screen brightness by its luma

    let screen_contribution = (current_brightness as f32 / FLOAT_MAX_PERCENT * FLOAT_MAX_COLOR) * SCREEN_REFLECTION_FACTOR;

    let adjusted_luma = raw_luma - screen_contribution;

    println!("[Processor] Raw Luma: {:.2} - Screen Contribution: {:.2} = Adjusted Luma: {:.2}",
        raw_luma, screen_contribution, adjusted_luma);

    Ok(adjusted_luma.max(0.0) as f32)
}

fn compute_luma(image: &RgbImage) -> Result<u8> {
    let raw_luma = compute_raw_luma(image)?;
    let adj_luma = adjusted_luma(raw_luma)?;

    Ok(adj_luma as u8)
}

fn get_screen_brightness() -> Result<u8> {
    for device in brightness::blocking::brightness_devices() {
        if let Ok(dev) = device {
            return Ok(dev.get()? as u8);
        }
    }
    Err(anyhow!("No primary brightness device found"))
}

fn set_screen_brightness(percent: u32) -> Result<()> {
    // set the brightness for the first (primary) brightness device
    for device in brightness::blocking::brightness_devices() {
        if let Ok(dev) = device {
            dev.set(percent)?;
            return Ok(());
        }
    }

    // no device was found iterating on ALL the devices
    Err(anyhow!("No primary brightness device found"))
}

pub fn auto_brightness(camera: &mut Camera) -> Result<()> {
    let frame_image = capture_frame(camera)?;

    let avg_luma = compute_luma(&frame_image)?;

    let percent_brightness = (avg_luma as f32 / FLOAT_MAX_COLOR * FLOAT_MAX_PERCENT) as u32;
    println!("[Processor] Luma: {}/{} -> Setting: {}%", avg_luma, FLOAT_MAX_COLOR, percent_brightness);
    set_screen_brightness(percent_brightness)?;

    Ok(())
}
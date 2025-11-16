use anyhow::Result;
use brightness::blocking::BrightnessDevice;
use image::ImageBuffer;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera
};

pub fn setup_camera() -> Result<Camera> {
    // Get the first camera
    let index = CameraIndex::Index(0);
    let fmt_requested = RequestedFormat::new<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    Camera::new(index, fmt_requested).map_err(|e| e.into())
}
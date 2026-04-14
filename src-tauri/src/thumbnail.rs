use std::path::PathBuf;
use std::fs;
use image::{
    DynamicImage, 
    imageops::{
        resize,
        FilterType
    }, 
    GenericImageView
};
use std::process::Command;
use crate::storage::{thumb_dir};

pub struct ThumbnailManager {}

impl ThumbnailManager {
    pub fn new() -> Self {
        Self {}
    }

    fn generate_thumbnail_filename(&self, original_path: &PathBuf) -> PathBuf {
        let filename = original_path.file_stem().unwrap_or_default().to_string_lossy();
        // We always save as PNG for thumbnails to handle both videos and images consistently
        PathBuf::from(format!("thumb_{}.png", filename))
    }

    pub fn get_thumbnail_path(&self, original_path: &PathBuf) -> PathBuf {
        let filename = self.generate_thumbnail_filename(original_path);
        thumb_dir().join(filename)
    }

    pub fn create_thumbnail(
        &self,
        original_path: &PathBuf,
        is_video: bool,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let thumb_path = self.get_thumbnail_path(original_path);
        
        if thumb_path.exists() {
            return Ok(thumb_path);
        }

        let img = if is_video {
            self.extract_video_frame(original_path)?
        } else {
            image::open(original_path)?
        };

        let (original_width, original_height) = img.dimensions();
        let max_width = 320;
        let max_height = 180;
        
        let ratio = original_width as f32 / original_height as f32;
        let (new_width, new_height) = if original_width > original_height {
            let height = (max_width as f32 / ratio).min(max_height as f32) as u32;
            (max_width.min(original_width), height)
        } else {
            let width = (max_height as f32 * ratio).min(max_width as f32) as u32;
            (width, max_height.min(original_height))
        };

        let resized = resize(&img, new_width, new_height, FilterType::Nearest);
        let resized_image = DynamicImage::ImageRgba8(resized);
        
        resized_image.save(&thumb_path)?;
        
        Ok(thumb_path)
    }

    fn extract_video_frame(
        &self,
        video_path: &PathBuf,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let temp_frame = std::env::temp_dir().join(format!(
            "frame_{}.png",
            video_path.file_stem().unwrap_or_default().to_string_lossy()
        ));

        let output = Command::new("ffmpeg")
            .args(&[
                "-i",
                video_path.to_str().ok_or("Invalid path")?,
                "-vf",
                "select=eq(n\\,0)",
                "-q:v",
                "2",
                "-vframes", "1",
                temp_frame.to_str().ok_or("Invalid temp path")?,
            ])
            .output()?;

        if !output.status.success() {
            let _ = fs::remove_file(&temp_frame);
            return Err("Failed to extract video frame with ffmpeg. Is it installed?".into());
        }

        let img = image::open(&temp_frame)?;
        let _ = fs::remove_file(&temp_frame);
        Ok(img)
    }
}

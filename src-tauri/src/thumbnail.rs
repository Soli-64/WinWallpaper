use crate::storage::thumb_dir;
use image::DynamicImage;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct ThumbnailManager {}

impl ThumbnailManager {
    pub fn new() -> Self {
        Self {}
    }

    // Generate thumbnail filename w/ prefix (e.g. "thumb_image.png")
    fn generate_thumbnail_filename(&self, original_path: &PathBuf) -> PathBuf {
        let filename = original_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        PathBuf::from(format!("thumb_{}.png", filename))
    }

    // Get full path of thumbnail from image path
    pub fn get_thumbnail_path(&self, original_path: &PathBuf) -> PathBuf {
        let filename = self.generate_thumbnail_filename(original_path);
        thumb_dir().join(filename)
    }

    //
    // Generate thumbnail from image path (with optional max width/height)
    //
    pub fn create_thumbnail(
        &self,
        original_path: &PathBuf,
        is_video: bool,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let thumb_path = self.get_thumbnail_path(original_path);

        if thumb_path.exists() {
            return Ok(thumb_path);
        }

        let max_width = 320;
        let max_height = 180;

        if is_video {
            let img = self.extract_video_frame(original_path, max_width, max_height)?;
            img.save(&thumb_path)?;
        } else {
            // .thumbnail() is more memory-efficient than loading + resizing manually
            let img = image::open(original_path)?;
            let thumb = img.thumbnail(max_width, max_height);
            thumb.save(&thumb_path)?;
        }

        Ok(thumb_path)
    }

    // 
    // Extract single frame from video using ffmpeg (scalable)
    // Used for thumbnail generation
    // 
    fn extract_video_frame(
        &self,
        video_path: &PathBuf,
        width: u32,
        height: u32,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let temp_frame = std::env::temp_dir().join(format!(
            "frame_{}.png",
            video_path.file_stem().unwrap_or_default().to_string_lossy()
        ));

        // ffmpeg scaling to save CPU/RAM
        let scale_filter = format!(
            "scale={}:{}:force_original_aspect_ratio=decrease",
            width, height
        );

        let output = Command::new("ffmpeg")
            .args(&[
                "-i",
                video_path.to_str().ok_or("Invalid path")?,
                "-vf",
                &format!("select=eq(n\\,0),{}", scale_filter),
                "-q:v",
                "2",
                "-vframes",
                "1",
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

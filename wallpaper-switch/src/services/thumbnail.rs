
use std::path::PathBuf;
use std::fs;
use image::{
    DynamicImage, 
    imageops::{
        resize,
        FilterType
    }, 
    GenericImageView};
use rayon::prelude::*;
use super::storage::{
    thumb_dir,
    list_files_recursive, 
    wallpapers_dir
};
use std::process::Command;

fn is_video_format(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return ["mp4", "avi", "webm", "mov", "mkv", "flv"].contains(&ext_str.as_str());
    }
    false
}

pub struct ThumbnailManager {}

impl ThumbnailManager {

    pub fn new() -> Self {
        Self {}
    }

    fn generate_thumbnail_filename(&self, original_path: &PathBuf) -> PathBuf {
        let filename = original_path.file_stem().unwrap_or_default().to_string_lossy();
        let extension = original_path.extension().unwrap_or_default().to_string_lossy();

        PathBuf::from(format!("thumb_{}.{}", filename, extension))
    }

    pub fn get_thumbnail_path(&self, original_path: &PathBuf) -> PathBuf {
        let filename = self.generate_thumbnail_filename(original_path);
        thumb_dir().join(filename)
    }

    pub fn resize_image(
        &self,
        input_path: &PathBuf,
        max_width: u32,
        max_height: u32,
        filter: FilterType,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let img = image::open(input_path)?;
        let (original_width, original_height) = img.dimensions();
        
        let ratio = original_width as f32 / original_height as f32;
        let (new_width, new_height) = if original_width > original_height {
            let height = (max_width as f32 / ratio).min(max_height as f32) as u32;
            (max_width.min(original_width), height)
        } else {
            let width = (max_height as f32 * ratio).min(max_width as f32) as u32;
            (width, max_height.min(original_height))
        };

        let resized = resize(&img, new_width, new_height, filter);
        Ok(DynamicImage::ImageRgba8(resized))
    }

    pub fn extract_video_frame(
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
                temp_frame.to_str().ok_or("Invalid temp path")?,
            ])
            .output()?;

        if !output.status.success() {
            let _ = fs::remove_file(&temp_frame);
            return Err("Failed to extract video frame with ffmpeg".into());
        }

        let img = image::open(&temp_frame)?;
        
        let _ = fs::remove_file(&temp_frame);

        Ok(img)
    }

    pub fn create_thumbnail(
        &self,
        original_path: &PathBuf,
        max_width: u32,
        max_height: u32,
        filter :FilterType,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let thumb_path = self.get_thumbnail_path(original_path);
        
        if thumb_path.exists() {
            return Ok(thumb_path);
        }

        let thumbnail = self.resize_image(original_path, max_width, max_height, filter)?;
        
        thumbnail.save(&thumb_path)?;
        
        Ok(thumb_path)
    }

    /// Create a thumbnail from a video file
    pub fn create_video_thumbnail(
        &self,
        video_path: &PathBuf,
        max_width: u32,
        max_height: u32,
        filter: FilterType,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let thumb_path = self.get_thumbnail_path(video_path);
        
        if thumb_path.exists() {
            return Ok(thumb_path);
        }

        // Extract first frame from video
        let frame = self.extract_video_frame(video_path)?;
        
        // Resize the frame
        let (original_width, original_height) = frame.dimensions();
        let ratio = original_width as f32 / original_height as f32;
        let (new_width, new_height) = if original_width > original_height {
            let height = (max_width as f32 / ratio).min(max_height as f32) as u32;
            (max_width.min(original_width), height)
        } else {
            let width = (max_height as f32 * ratio).min(max_width as f32) as u32;
            (width, max_height.min(original_height))
        };

        let resized = resize(&frame, new_width, new_height, filter);
        let resized_image = DynamicImage::ImageRgba8(resized);
        
        // Convert to PNG since videos keep their extension in thumbs
        let png_thumb_path = thumb_path.with_extension("png");
        resized_image.save(&png_thumb_path)?;
        
        Ok(png_thumb_path)
    }

    pub fn create_preview_thumbnail(&self, original_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
        println!("Creating preview thumbnail for {:?}", original_path);
        self.create_thumbnail(
            original_path,
            320,  
            180,  
            FilterType::Nearest
        )
    }

    pub fn cleanup_orphaned_thumbnails(&self) -> () {
        // Collect all valid thumbnails from image and video files
        let mut existing_thumbnails: Vec<PathBuf> = list_files_recursive(wallpapers_dir(), Some(1), Some(&["png","jpg","jpeg"]))
            .unwrap_or_default()
            .iter()
            .map(|path| self.generate_thumbnail_filename(path))
            .collect();

        // Add video thumbnails
        let video_extensions = ["mp4", "avi", "webm", "mov", "mkv", "flv"];
        let video_thumbnails: Vec<PathBuf> = list_files_recursive(wallpapers_dir(), Some(1), Some(&video_extensions))
            .unwrap_or_default()
            .iter()
            .map(|path| {
                let filename = path.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("thumb_{}.png", filename))
            })
            .collect();
        
        existing_thumbnails.extend(video_thumbnails);

        for entry in fs::read_dir(&thumb_dir()).unwrap() {
            let entry = entry.unwrap();
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();
            
            if !filename_str.starts_with("thumb_") {
                continue;
            }

            if !existing_thumbnails.contains(&PathBuf::from(filename_str.as_ref())) {
                println!("Removing orphaned thumbnail: {:?}", entry.path());
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    pub fn generate_fast_thumbs(&self) {
        // Generate thumbnails for images
        let image_paths = list_files_recursive(wallpapers_dir(), Some(1), Some(&["jpg", "png", "jpeg"])) 
            .expect("Failed to list wallpaper files");
        println!("Generating thumbnails for {} image files", image_paths.len());
        image_paths.par_iter().for_each(|p| {
            let _ = self.create_preview_thumbnail(p);
        });

        // Generate thumbnails for videos
        let video_extensions = ["mp4", "avi", "webm", "mov", "mkv", "flv"];
        let video_paths = list_files_recursive(wallpapers_dir(), Some(1), Some(&video_extensions))
            .expect("Failed to list video files");
        println!("Generating thumbnails for {} video files", video_paths.len());
        video_paths.par_iter().for_each(|p| {
            if let Err(e) = self.create_video_thumbnail(p, 320, 180, FilterType::Nearest) {
                eprintln!("Failed to create video thumbnail for {:?}: {}", p, e);
            }
        });
    } 

}

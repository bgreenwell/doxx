use anyhow::Result;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// Type aliases to simplify complex return types
type ImageList<'a> = Vec<(&'a String, &'a PathBuf)>;
type ExtractedImages = Vec<(String, PathBuf)>;

/// Manages extraction of images from DOCX files
#[derive(Debug)]
pub struct ImageExtractor {
    temp_dir: PathBuf,
    extracted_images: HashMap<String, PathBuf>, // relationship_id -> temp_file_path
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Tiff,
}

impl ImageFormat {
    pub fn from_filename(filename: &str) -> Option<Self> {
        let extension = Path::new(filename).extension()?.to_str()?.to_lowercase();

        match extension.as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            _ => None,
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }
}

impl ImageExtractor {
    /// Create a new image extractor with a temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("doxx_images");
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            temp_dir,
            extracted_images: HashMap::new(),
        })
    }

    /// Extract all images from a DOCX file
    pub fn extract_images_from_docx(&mut self, docx_path: &Path) -> Result<()> {
        let file = File::open(docx_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Look for images in the word/media/ folder
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = file.name().to_string(); // Clone the name to avoid borrow issues

            // Check if this is an image file in the media folder
            if outpath.starts_with("word/media/") && self.is_image_file(&outpath) {
                let filename = Path::new(&outpath)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                // Create a unique temp file path
                let temp_file_path = self.temp_dir.join(filename);

                // Read the image data
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                // Write to temp file
                let mut temp_file = File::create(&temp_file_path)?;
                temp_file.write_all(&buffer)?;

                // Store the mapping (we'll enhance this with proper relationship parsing later)
                let rel_id = filename.to_string(); // Simplified for now
                self.extracted_images.insert(rel_id, temp_file_path);
            }
        }

        println!(
            "Extracted {} images to {}",
            self.extracted_images.len(),
            self.temp_dir.display()
        );
        Ok(())
    }

    /// Get image data by relationship ID
    pub fn get_image_data(&self, rel_id: &str) -> Result<Vec<u8>> {
        if let Some(path) = self.extracted_images.get(rel_id) {
            Ok(fs::read(path)?)
        } else {
            anyhow::bail!("Image not found: {}", rel_id)
        }
    }

    /// Get the path to an extracted image
    pub fn get_image_path(&self, rel_id: &str) -> Option<&PathBuf> {
        self.extracted_images.get(rel_id)
    }

    /// List all extracted images
    pub fn list_images(&self) -> ImageList {
        self.extracted_images.iter().collect()
    }

    /// Get all extracted images as a vector of (rel_id, path) pairs
    pub fn get_extracted_images(&self) -> ExtractedImages {
        self.extracted_images
            .iter()
            .map(|(rel_id, path)| (rel_id.clone(), path.clone()))
            .collect()
    }

    /// Get all extracted images sorted by filename for consistent ordering
    pub fn get_extracted_images_sorted(&self) -> ExtractedImages {
        let mut images: ExtractedImages = self
            .extracted_images
            .iter()
            .map(|(rel_id, path)| (rel_id.clone(), path.clone()))
            .collect();

        // Sort by filename to ensure consistent ordering
        images.sort_by(|a, b| a.0.cmp(&b.0));
        images
    }

    /// Clean up temporary files
    pub fn cleanup(&self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    /// Check if a file is an image based on its extension
    fn is_image_file(&self, filename: &str) -> bool {
        ImageFormat::from_filename(filename).is_some()
    }
}

impl Drop for ImageExtractor {
    fn drop(&mut self) {
        // Don't automatically clean up temp files - let them persist
        // for the lifetime of the application. The OS will clean them up
        // when the temp directory is cleared, or users can manually clean up.
        // let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        assert!(matches!(
            ImageFormat::from_filename("image.png"),
            Some(ImageFormat::Png)
        ));
        assert!(matches!(
            ImageFormat::from_filename("photo.jpg"),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::from_filename("photo.jpeg"),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::from_filename("animation.gif"),
            Some(ImageFormat::Gif)
        ));
        assert!(matches!(
            ImageFormat::from_filename("bitmap.bmp"),
            Some(ImageFormat::Bmp)
        ));
        assert!(matches!(
            ImageFormat::from_filename("scan.tiff"),
            Some(ImageFormat::Tiff)
        ));
        assert!(ImageFormat::from_filename("document.txt").is_none());
    }

    #[test]
    fn test_image_extractor_creation() {
        let extractor = ImageExtractor::new().unwrap();
        assert!(extractor.temp_dir.exists());
        assert!(extractor.extracted_images.is_empty());
    }
}

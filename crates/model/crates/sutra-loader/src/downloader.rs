use crate::error::{LoaderError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Configuration for model downloads
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Base URL for model repository
    pub base_url: String,

    /// Local cache directory
    pub cache_dir: PathBuf,

    /// Verify checksum after download
    pub verify_checksum: bool,

    /// Show progress bar
    pub show_progress: bool,

    /// Number of retry attempts
    pub max_retries: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("sutraworks")
            .join("models");

        Self {
            base_url: "https://huggingface.co".to_string(),
            cache_dir,
            verify_checksum: true,
            show_progress: true,
            max_retries: 3,
        }
    }
}

/// Model downloader for fetching pre-trained weights
pub struct ModelDownloader {
    config: DownloadConfig,
    client: Client,
}

impl ModelDownloader {
    /// Create a new model downloader with the given configuration
    pub fn new(config: DownloadConfig) -> Result<Self> {
        // Ensure cache directory exists
        fs::create_dir_all(&config.cache_dir)?;

        let client = Client::builder()
            .user_agent("sutraworks-model/0.1.0")
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self { config, client })
    }

    /// Create a downloader with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(DownloadConfig::default())
    }

    /// Download a model file from HuggingFace
    ///
    /// # Arguments
    /// * `repo` - Repository name (e.g., "meta-llama/Llama-2-7b-hf")
    /// * `filename` - File to download (e.g., "model.safetensors")
    /// * `revision` - Git revision (branch/tag/commit), defaults to "main"
    pub fn download_hf(
        &self,
        repo: &str,
        filename: &str,
        revision: Option<&str>,
    ) -> Result<PathBuf> {
        let revision = revision.unwrap_or("main");

        // Construct URL
        let url = format!(
            "{}/{}/resolve/{}/{}",
            self.config.base_url, repo, revision, filename
        );

        // Determine local path
        let local_path = self
            .config
            .cache_dir
            .join(repo)
            .join(revision)
            .join(filename);

        // Check if already downloaded
        if local_path.exists() {
            println!("Found cached model at: {}", local_path.display());
            return Ok(local_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Download with retries
        let mut attempts = 0;
        loop {
            match self.download_file(&url, &local_path) {
                Ok(_) => break,
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        return Err(e);
                    }
                    eprintln!(
                        "Download failed (attempt {}/{}): {}",
                        attempts, self.config.max_retries, e
                    );
                    std::thread::sleep(std::time::Duration::from_secs(2u64.pow(attempts)));
                }
            }
        }

        Ok(local_path)
    }

    /// Download a file from a URL to a local path
    fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        println!("Downloading: {}", url);

        let mut response = self.client.get(url).send()?;

        if !response.status().is_success() {
              return Err(LoaderError::Download(
                response.error_for_status().unwrap_err(),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Create progress bar if enabled
        let progress = if self.config.show_progress && total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-")
            );
            Some(pb)
        } else {
            None
        };

        // Download to temporary file first
        let temp_path = dest.with_extension("tmp");
        let mut file = File::create(&temp_path)?;
        let mut hasher = Sha256::new();

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = response.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            file.write_all(&buffer[..bytes_read])?;
            hasher.update(&buffer[..bytes_read]);

            if let Some(pb) = &progress {
                pb.inc(bytes_read as u64);
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Download complete");
        }

        // Move to final location
        fs::rename(&temp_path, dest)?;

        println!("Saved to: {}", dest.display());

        // Compute checksum
        let checksum = format!("{:x}", hasher.finalize());
        println!("SHA256: {}", checksum);

        Ok(())
    }

    /// Verify file checksum
    pub fn verify_checksum(&self, path: &Path, expected: &str) -> Result<()> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();

        io::copy(&mut file, &mut hasher)?;
        let actual = format!("{:x}", hasher.finalize());

        if actual != expected {
            return Err(LoaderError::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }

        Ok(())
    }

    /// Get path to cached model
    pub fn cached_path(&self, repo: &str, filename: &str, revision: Option<&str>) -> PathBuf {
        let revision = revision.unwrap_or("main");
        self.config
            .cache_dir
            .join(repo)
            .join(revision)
            .join(filename)
    }

    /// Clear cache for a specific model
    pub fn clear_cache(&self, repo: &str) -> Result<()> {
        let repo_dir = self.config.cache_dir.join(repo);
        if repo_dir.exists() {
            fs::remove_dir_all(repo_dir)?;
        }
        Ok(())
    }
}

// Use std::io::Read trait
use std::io::Read;

// Simple dirs crate replacement
mod dirs {
    use std::path::PathBuf;

    pub fn cache_dir() -> Option<PathBuf> {
        if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .ok()
                .map(|home| PathBuf::from(home).join("Library/Caches"))
        } else if cfg!(target_os = "linux") {
            std::env::var("XDG_CACHE_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME")
                        .ok()
                        .map(|home| PathBuf::from(home).join(".cache"))
                })
        } else if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let config = DownloadConfig::default();
        let downloader = ModelDownloader::new(config);
        assert!(downloader.is_ok());
    }

    #[test]
    fn test_cached_path() {
        let downloader = ModelDownloader::with_defaults().unwrap();
        let path = downloader.cached_path("test/model", "weights.safetensors", Some("main"));
        assert!(path.to_string_lossy().contains("test/model"));
        assert!(path.to_string_lossy().contains("main"));
        assert!(path.to_string_lossy().contains("weights.safetensors"));
    }
}

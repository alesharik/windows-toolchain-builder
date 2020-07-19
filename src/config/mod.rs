use regex::Regex;
use std::path::PathBuf;

pub mod clap;

/// Application configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Package name which will be used as root to download all stuff
    pub package: String,
    /// Repository base URL (will be appended with architecture to get repo URL)
    pub repository: String,
    /// Repository name (required to download {}.db.tar.gz file)
    pub repository_name: String,
    /// Wanted architecture. Will be used with repository base URL to crete repo URL
    pub architecture: String,
    /// Download/extract parallel task count
    pub parallelism: u32,
    /// Match files/folders to exclude them from output
    pub exclude: Vec<Regex>,
    /// Match files/folders to include them into output. Have less priority than `exclude`. Will match
    /// all packages if empty.
    pub include: Vec<Regex>,
    /// Output folder path. Will be created automatically with all parents, if not exist
    pub output_folder: PathBuf,
}

impl Config {
    pub fn repository_url(&self) -> String {
        self.repository.clone() + "/" + &self.architecture
    }
}

pub trait IntoConfig {
    fn to_config(&self) -> Config;
}
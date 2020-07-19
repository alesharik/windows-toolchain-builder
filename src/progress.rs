use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::time::Duration;
use archlinux_repo::Package;

pub struct Progress {
    progress: Arc<MultiProgress>
}

impl Progress {
    pub fn new() -> Self {
        let progress = Arc::new(MultiProgress::new());
        let progress_exec = progress.clone();
        std::thread::spawn(move || {
            loop {
                progress_exec.join().unwrap();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        Progress { progress }
    }

    pub fn repo(&self) -> RepoLoadProgress {
        RepoLoadProgress {
            progress: self.progress.clone(),
            repo_load_progress: None
        }
    }

    pub fn tree(&self) -> TreeBuildProgress {
        TreeBuildProgress::new(self.progress.clone())
    }

    pub fn package_download(&self, name: &str) -> PackageDownloadProgress {
        PackageDownloadProgress::new(self.progress.as_ref(), name)
    }

    pub fn package_extract(&self, name: &str) -> PackageExtractProgress {
        PackageExtractProgress::new(self.progress.as_ref(), name)
    }
}

pub struct PackageDownloadProgress {
    progress: ProgressBar,
    name: String
}

impl PackageDownloadProgress {
    fn new(progress: &MultiProgress, package: &str) -> Self {
        let bar = progress.add(ProgressBar::new(1));
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} Downloading {wide_msg}: [{elapsed_precise}] [{bar:80.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-")
        );
        bar.set_message(package);
        PackageDownloadProgress { progress: bar, name: package.to_owned() }
    }

    pub fn chunk(&self, pos: u64, max: u64) {
        self.progress.set_length(max);
        self.progress.set_position(pos);
    }

    pub fn complete(self) {
        let msg = format!("Package {} downloaded", &self.name);
        self.progress.println(msg);
        self.progress.finish_and_clear();
    }
}

pub struct PackageExtractProgress {
    progress: ProgressBar,
    name: String
}

impl PackageExtractProgress {
    fn new(progress: &MultiProgress, package: &str) -> Self {
        let bar = progress.add(ProgressBar::new(1));
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} Extracting {wide_msg}: [{elapsed_precise}] [{bar:80.cyan/blue}] {pos}/{len} ({eta})")
                .progress_chars("#>-")
        );
        bar.set_message(package);
        PackageExtractProgress { progress: bar, name: package.to_owned() }
    }

    pub fn set_count(&self, count: usize) {
        self.progress.set_length(count as u64);
    }

    pub fn file(&self, file: &str) {
        self.progress.set_message(file);
        self.progress.inc(1);
    }

    pub fn complete(self) {
        let msg = format!("Package {} extracted", &self.name);
        self.progress.println(msg);
        self.progress.finish_and_clear();
    }
}

pub struct RepoLoadProgress {
    progress: Arc<MultiProgress>,
    repo_load_progress: Option<ProgressBar>
}

impl RepoLoadProgress {
    pub fn report(&mut self, progress: archlinux_repo::Progress) {
        let multi_progress = self.progress.as_ref();
        match progress {
            archlinux_repo::Progress::LoadingDb => {}
            archlinux_repo::Progress::LoadingFilesMetadata => {}
            archlinux_repo::Progress::LoadingDbChunk(current, size) => {
                let progress = self.repo_load_progress
                    .get_or_insert_with(|| {
                        let p = multi_progress.add(if let Some(max) = size {
                            ProgressBar::new(max)
                        } else {
                            ProgressBar::new_spinner()
                        });
                        p.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green} {wide_msg}: [{elapsed_precise}] [{bar:80.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                                .progress_chars("#>-")
                        );
                        p.set_message("Loading repository");
                        p
                    });
                progress.set_position(current);
                if let Some(s) = size  {
                    if s == current {
                        progress.println("Repository loaded");
                        progress.finish_and_clear();
                        self.repo_load_progress = None
                    }
                }
            }
            archlinux_repo::Progress::ReadingDbFile(file) => {
                let progress = self.repo_load_progress
                    .get_or_insert_with(|| {
                        let p = multi_progress.add(ProgressBar::new_spinner());
                        p.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green} {wide_msg}: [{bar:80.cyan/blue}]")
                                .progress_chars("#>-")
                        );
                        p
                    });
                let msg = format!("Reading file {}", file);
                progress.set_message(&msg);
            }
            archlinux_repo::Progress::ReadingDbDone => {
                if let Some(progress) = self.repo_load_progress.as_ref() {
                    progress.println("Repository reading complete");
                    progress.finish_and_clear();
                }
                self.repo_load_progress = None
            }
            archlinux_repo::Progress::LoadingFilesMetadataChunk(current, size) => {
                let progress = self.repo_load_progress
                    .get_or_insert_with(|| {
                        let p = multi_progress.add(if let Some(max) = size {
                            ProgressBar::new(max)
                        } else {
                            ProgressBar::new_spinner()
                        });
                        p.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green} {wide_msg}: [{elapsed_precise}] [{bar:80.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                                .progress_chars("#>-")
                        );
                        p.set_message("Loading files metadata");
                        p
                    });
                progress.set_length(current);
                if let Some(s) = size  {
                    if s == current {
                        progress.println("Files metadata loaded");
                        progress.finish_and_clear();
                        self.repo_load_progress = None
                    }
                }
            }
            archlinux_repo::Progress::ReadingFilesMetadataFile(file) => {
                let progress = self.repo_load_progress
                    .get_or_insert_with(|| {
                        let p = multi_progress.add(ProgressBar::new_spinner());
                        p.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green} {wide_msg}: [{bar:80.cyan/blue}]")
                                .progress_chars("#>-")
                        );
                        p
                    });
                let msg = format!("Reading file {}", file);
                progress.set_message(&msg);
            }
            archlinux_repo::Progress::ReadingFilesDone => {
                if let Some(progress) = self.repo_load_progress.as_ref() {
                    progress.println("Repository files metadata reading complete");
                    progress.finish_and_clear();
                }
                self.repo_load_progress = None
            }
        }
    }
}

pub struct TreeBuildProgress {
    progress_bar: ProgressBar
}

impl TreeBuildProgress {
    fn new(progress: Arc<MultiProgress>) -> Self {
        let progress_bar = progress.add(ProgressBar::new_spinner());
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}: [{elapsed_precise}] [{bar:80.cyan/blue}]")
                .progress_chars("#>-")
        );
        progress_bar.set_message("Building tree");
        TreeBuildProgress { progress_bar }
    }

    pub fn index(&self, package: &Package) {
        let msg = format!("Indexing {}", package.name);
        self.progress_bar.set_message(&msg);
    }

    pub fn done(self) {
        self.progress_bar.println("Tree built");
        self.progress_bar.finish_and_clear();
    }
}
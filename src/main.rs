mod config;
mod progress;

use archlinux_repo::{RepositoryBuilder, Package, Repository};
use std::sync::RwLock;
use crate::progress::Progress;
use std::path::PathBuf;
use std::error::Error;
use tokio::fs::OpenOptions;
use futures::StreamExt;
use crate::config::Config;
use compress_tools::{list_archive_files, uncompress_archive_file};
use std::io::{Write, Cursor};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProgramError {
    PackageNotFound(String),
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::PackageNotFound(name) => write!(f, "Package {} not found", name)
        }
    }
}

impl Error for ProgramError {}

struct Program {
    config: Config,
    progress: Progress,
    output: PathBuf,
    repository: Repository,
}

impl Program {
    pub async fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let progress = Progress::new();

        let output = config.output_folder.clone();
        tokio::fs::create_dir_all(&output).await?;

        let repo_progress = RwLock::new(progress.repo());
        let repository = RepositoryBuilder::new(&config.repository_name, &config.repository_url())
            .progress_listener(Box::new(move |p| repo_progress.write().unwrap().report(p)))
            .load()
            .await?;

        Ok(Program {
            config,
            output,
            progress,
            repository
        })
    }

    pub async fn run(&self, package: &str) -> Result<(), Box<dyn Error>> {
        let package = self.repository[package].to_owned();
        let tree = self.build_package_tree(package)?;
        let mut download_stream = futures::stream::iter(tree.iter().map(|package| self.process_package(package)))
            .buffer_unordered(self.config.parallelism as usize);
        loop {
            let (result, stream) = download_stream.into_future().await;
            download_stream = stream;
            if result.is_none() {
                break;
            }
        }
        Ok(())
    }

    async fn process_package(&self, package: &Package) -> Result<(), Box<dyn Error>> {
        let archive = self.download_package(&package).await?;
        self.extract_package(archive, &package).await?;
        Ok(())
    }

    async fn extract_package(&self, archive: Vec<u8>, package: &Package) -> Result<(), Box<dyn Error>> {
        use tokio::io::AsyncWriteExt;

        let progress = self.progress.package_extract(&package.name);
        let files = list_archive_files(&archive[..])?;
        progress.set_count(files.len());
        for file in files.iter() {
            progress.file(file);
            if file.ends_with('/') || file.starts_with('.') {
                continue;
            }
            if self.config.exclude.iter().any(|regex| regex.is_match(file)) {
                continue;
            }
            if self.config.include.is_empty() || self.config.include.iter().any(|regex| regex.is_match(file)) {
                let mut vec = Vec::<u8>::new();
                let buf = Cursor::new(&mut vec);
                uncompress_archive_file(&archive[..], buf, file)?;
                let path = self.output.join(file);
                tokio::fs::create_dir_all(path.parent().unwrap()).await?;
                let mut fs_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&path).await?;
                fs_file.write_all(&vec[..]).await?;
                fs_file.flush().await?;
            }
        }
        progress.complete();
        Ok(())
    }

    async fn download_package(&self, package: &Package) -> Result<Vec<u8>, Box<dyn Error>> {
        let progress = self.progress.package_download(&package.name);
        let mut buf = Vec::new();
        let mut response = self.repository.request_package(&package.name).await?;
        let mut bytes_read: u64 = 0;
        let length = response.content_length().unwrap();
        while let Some(chunk) = response.chunk().await? {
            buf.write_all(&chunk[..])?;
            bytes_read += chunk.len() as u64;
            progress.chunk(bytes_read, length);
        }
        progress.complete();
        Ok(buf)
    }

    fn build_package_tree(&self, package: Package) -> Result<Vec<Package>, ProgramError> {
        let progress = self.progress.tree();
        let mut tree = Vec::<Package>::new();
        tree.push(package);
        loop {
            let mut modified = false;
            let mut patch = Vec::<Package>::new();
            for item in tree.iter() {
                progress.index(item);
                if let Some(deps) = item.depends.as_ref() {
                    for dependency in deps {
                        let package = self.repository.get_package_by_name(&dependency.name)
                            .ok_or_else(|| ProgramError::PackageNotFound(dependency.name.clone()))?;
                        if !tree.contains(package) && !patch.contains(package) {
                            patch.push(package.to_owned());
                            modified = true;
                        }
                    }
                }
            }
            tree.append(&mut patch);
            if !modified {
                break
            }
        }
        progress.done();
        Ok(tree)
    }
}

#[tokio::main(core_threads = 8, max_threads = 16)]
async fn main() {
    let config = config::clap::config();
    let program = Program::new(config.clone()).await.unwrap();
    program.run(&config.package).await.unwrap();
}
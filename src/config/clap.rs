//! This module provides configuration from CLI arguments
use clap::{ArgMatches, App, Arg};
use crate::config::{IntoConfig, Config};
use std::str::FromStr;
use regex::Regex;
use std::path::PathBuf;

impl IntoConfig for ArgMatches<'static> {
    fn to_config(&self) -> Config {
        let cpu_count = num_cpus::get().to_string();
        Config {
            package: self.value_of("package").unwrap().to_string(),
            repository: self.value_of("repository").unwrap().to_string(),
            repository_name: self.value_of("repository-name").unwrap().to_string(),
            architecture: self.value_of("architecture").unwrap().to_string(),
            parallelism: u32::from_str(&self.value_of("parallelism").unwrap_or(&cpu_count).to_string()).unwrap(),
            exclude: self.values_of("exclude").map(|v| v.map(|val| Regex::new(val).unwrap()).collect()).unwrap_or(Vec::new()),
            include: self.values_of("include").map(|v| v.map(|val| Regex::new(val).unwrap()).collect()).unwrap_or(Vec::new()),
            output_folder: PathBuf::from(self.value_of("output").unwrap())
        }
    }
}

fn args() -> Box<ArgMatches<'static>> {
    Box::new(
        App::new("windows-toolchain-builder")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Aleksei Arsenev <alesharik4@gmail.com>")
            .arg(
                Arg::with_name("package")
                    .index(1)
                    .help("Package name")
                    .required(true)
            )
            .arg(
                Arg::with_name("repository")
                    .short("r")
                    .long("repository")
                    .value_name("REPOSITORY")
                    .help("Address to package repository")
                    .takes_value(true)
                    .default_value("http://repo.msys2.org/mingw")
            )
            .arg(
                Arg::with_name("repository-name")
                    .short("n")
                    .long("reponame")
                    .value_name("REPOSITORY_NAME")
                    .help("Package repository name")
                    .takes_value(true)
                    .default_value("mingw64")
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .value_name("OUTPUT")
                    .help("Output folder")
                    .takes_value(true)
                    .default_value("./")
            )
            .arg(
                Arg::with_name("parallelism")
                    .short("p")
                    .value_name("PARALLELISM")
                    .help("Download/extract thread pool parallelism")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("exclude")
                    .short("e")
                    .value_name("EXCLUDE")
                    .help("Exclude files or folders by regex")
                    .multiple(true)
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("include")
                    .short("i")
                    .value_name("INCLUDE")
                    .help("Include files or folders by regex. Only files which matches regex will be included. All files which matches include and exclude regex will *not* be included")
                    .multiple(true)
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("architecture")
                    .short("a")
                    .long("arch")
                    .value_name("ARCH")
                    .help("Package architecture")
                    .takes_value(true)
                    .default_value("x86_64")
                    .validator(|arch| {
                        if arch == "x86_64" || arch == "i686" {
                            return Ok(());
                        }
                        Err(String::from(format!("Unknown architecture: \"{}\"", arch)))
                    })
            )
            .get_matches()
    )
}

/// Parse CLI arguments, deserialize them to configuration and return it.
/// Will panic when have illegal or insufficient arguments.
pub fn config() -> Config {
    args().to_config()
}
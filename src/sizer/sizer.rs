//! Sizer is a small tool used to find directories that are taking
//! up the most hard drive space.

use std::cmp::Ordering;
use std::env;
use std::fs::{self, DirEntry};
use std::path;
use std::io;

#[macro_use]
extern crate clap;
use clap::{App, Arg};


#[derive(Debug, Eq)]
struct DirectoryInfo {
    path: path::PathBuf,
    size: u64,
}

impl Ord for DirectoryInfo {
    fn cmp(&self, other: &DirectoryInfo) -> Ordering {
        self.size.cmp(&other.size)
    }
}

impl PartialOrd for DirectoryInfo {
    fn partial_cmp(&self, other: &DirectoryInfo) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DirectoryInfo {
    fn eq(&self, other: &DirectoryInfo) -> bool {
        self.size == other.size
    }
}

struct Searcher {
    root_dir: path::PathBuf,
    depth: u64,
}

impl Searcher {
    fn search(&self) -> Vec<DirectoryInfo> {
        let mut results = Vec::new();
        self.parse_dir(&self.root_dir, &mut results);
        results
    }

    fn parse_dir(&self, dir: &path::Path, dirs: &mut Vec<DirectoryInfo>) {
        let mut size_of_files = 0;
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let md = entry.metadata().unwrap();
            if md.is_file() {
                size_of_files = size_of_files + md.len();
            } else if md.is_dir() {
                let sub_dir = &entry.path();
                if Searcher::distance(&self.root_dir, sub_dir).unwrap() < self.depth {
                    self.parse_dir(sub_dir, dirs);
                } else {
                    let size = Searcher::size_of_dir(sub_dir);
                    let info = DirectoryInfo {
                        path: path::PathBuf::from(sub_dir),
                        size: size,
                    };
                    dirs.push(info);
                }
            }
        }
        let info = DirectoryInfo {
            path: path::PathBuf::from(dir),
            size: size_of_files,
        };
        dirs.push(info);
}

    /// Calculates the distance from `dir` to `root`
    fn distance(root: &path::Path, dir: &path::Path) -> Option<u64> {
        let mut distance = 0;
        let mut root_found = false;
        let mut ancestors = dir.ancestors();
        ancestors.next(); // Skip self
        for parent in ancestors {
            distance = distance + 1;
            if parent == root {
                root_found = true;
                break;
            }
        }

        if root_found {
            Some(distance)
        } else {
            None
        }
    }

    /// Calculates the size of all files within `dirs` sub directories
    fn size_of_dir(dir: &path::Path) -> u64 {
        let mut size = 0;
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let md = entry.metadata().unwrap();
            if md.is_file() {
                size = size + md.len();
            } else if md.is_dir() {
                size = size + Searcher::size_of_dir(&entry.path());
            }
        }

        size
    }
}

fn main() {
    let matches = App::new("Sizer")
                      .version(crate_version!())
                      .author("Cade Colvin <cade.colvin@gmail.com>")
                      .about("Finds the largest directories")
                      .arg(Arg::with_name("root")
                          .short("r")
                          .long("root-directory")
                          .takes_value(true)
                          .help("The root directory to begin sizing. Default to PWD."))
                      .arg(Arg::with_name("result_count")
                          .short("n")
                          .long("results")
                          .takes_value(true)
                          .help("The number of results to output. Defaults to 10."))
                      .arg(Arg::with_name("depth")
                          .short("d")
                          .long("depth")
                          .takes_value(true)
                          .help("The depth to aggregate the resuls at. Defaults to 5."))
                      .get_matches();

    let root_dir = match matches.value_of("root") {
        Some(m) => path::PathBuf::from(m),
        None => env::current_dir().unwrap()
    };
    
    let mut result_count: usize = match matches.value_of("result_count") {
        Some(m) => usize::from_str_radix(m, 10).unwrap(),
        None => 10,
    };

    let depth = match matches.value_of("depth") {
        Some(m) => u64::from_str_radix(m, 10).unwrap(),
        None => 5,
    };

    let searcher = Searcher {
        root_dir: root_dir,
        depth: depth,
    };

    let mut results = searcher.search();
    results.sort();


    if result_count > results.len() {
        result_count = results.len();
    }

    for info in results.iter().rev().take(result_count) {
        println!("Directory: {:?}, Size: {}", info.path, info.size);
    }
}
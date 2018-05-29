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
                      .get_matches();

    let root_dir = match matches.value_of("root") {
        Some(m) => path::PathBuf::from(m),
        None => env::current_dir().unwrap()
    };

    let mut dir_info:Vec<DirectoryInfo> = Vec::new();
    parse_sub_dirs(root_dir, &mut dir_info);
    dir_info.sort();

    let result_count: usize = match matches.value_of("result_count") {
        Some(m) => usize::from_str_radix(m, 10).unwrap(),
        None => 10,
    };

    if result_count > dir_info.len() {
        let result_count = dir_info.len();
    }

    for info in dir_info.iter().rev().take(result_count) {
        println!("{:?}", info);
    }
}

/// Recursively searches `dir` and fills `info` with
/// `DirectoryInfo` for each directory found.
fn parse_sub_dirs(dir: path::PathBuf, info: &mut Vec<DirectoryInfo>) {
    let mut total_size = 0;
    let sub_dirs = dir.read_dir().unwrap();

    for sub_dir in sub_dirs{
        let sub_dir_path = path::PathBuf::from(sub_dir.unwrap().path());
        let meta_data = fs::metadata(&sub_dir_path).unwrap();

        if meta_data.is_file() {
            total_size = total_size + meta_data.len();
        } else {
            parse_sub_dirs(sub_dir_path, info)
        }
    }
    let dir_info = DirectoryInfo {
        path: dir,
        size: total_size,
    };
    info.push(dir_info);
}
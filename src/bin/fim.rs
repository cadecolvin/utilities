//! Find IMproved is a customized version of the unix Find tool.
//! 
//! Fim is used to perform a basic regex search of filenames.
//! By default, fim will perform a recursive search starting with ./.
//! A start directory can be specified with the `directory` parameter however.


use std::env;
use std::path::{Path,PathBuf};
use std::process;

extern crate clap;
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

fn main() {
    let matches = App::new("Fim: Find IMproved")
                            .version("0.2.0")
                            .author("Cade Colvin <cade.colvin@gmail.com>")
                            .about("A customized version of the unix find command")
                            .arg(Arg::with_name("directory")
                                .short("d")
                                .long("directory")
                                .takes_value(true)
                                .help("The root directory to search. Defaults to PWD"))
                            .arg(Arg::with_name("pattern")
                                .index(1)
                                .required(true)
                                .help("The Regular Expression used in the search"))
                            .get_matches();

    // Specify the directory the search should start in
    let root_dir = match matches.value_of("directory") {
        Some(m) => PathBuf::from(m),
        None => env::current_dir().unwrap()
    };
    
    // Parse the passed text into a regex pattern
    let re_pattern = matches.value_of("pattern").unwrap();
    let re = match Regex::new(re_pattern) {
        Ok(r) => r,
        Err(_) => {
            println!("Unable to parse regex string");
            process::exit(1);
        }
    };

    let mut file_paths= Vec::new();
    get_file_paths(root_dir.as_path(), &mut file_paths);

    for f in file_paths{
        if let Some(file_name) = f.file_name() {
            if re.is_match(file_name.to_str().unwrap()) {
                println!("{}", f.to_str().unwrap());
            }
        }
    }
}

/// Recursively fills `file_paths` with the paths of files
/// found within `dir` and all sub directories
/// 
/// #Arguments
/// * `dir` - The top directory to begin pulling files from
/// * `file_paths` - The vector that the file paths will be
///                  appended to.
fn get_file_paths(dir: &Path, file_paths: &mut Vec<PathBuf>) {
    for entry in dir.read_dir().unwrap() {
        let path = entry.unwrap().path();

        if path.is_file() {
            file_paths.push(path);
        } else if path.is_dir() {
            get_file_paths(path.as_path(), file_paths);
        }
    }
}
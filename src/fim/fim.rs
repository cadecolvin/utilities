//! **F**ind **IM**proved is a customized version of the unix Find tool.
//! 
//! Fim is used to perform a basic regex search of filenames.
//! By default, fim will perform a recursive search starting with `./`. 
//! A start directory can be specified with the `--directory` parameter however.


use std::env;
use std::io;
use std::io::prelude::*;
use std::path::{self, Path, PathBuf};
use std::process;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate clap;
extern crate regex;
extern crate term;
use clap::{App, Arg};
use regex::Regex;


fn main() {
    let matches = App::new("Fim: Find IMproved")
                        .version(crate_version!())
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
    
    // Parse the `pattern` argument into a regex pattern
    let re_pattern = matches.value_of("pattern").unwrap();
    let re = match Regex::new(re_pattern) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("Unable to parse regex string");
            process::exit(1);
        }
    };

    let (tx, rx) = mpsc::channel();
    let progress = thread::spawn(move|| { show_progress(rx); });

    let mut file_paths= Vec::new();
    get_file_paths(root_dir.as_path(), &mut file_paths);

    tx.send(()).unwrap();
    progress.join().unwrap();

    // Setup stdout so that we can color the match in the output
    let mut stdout = term::stdout().unwrap();

    for f in file_paths{
        if let Some(file_name) = f.file_name() {
            let file_name_str = file_name.to_str().unwrap();
            if re.is_match(file_name_str) {
                if let Some(m) = re.find(file_name_str) {
                    // Split the filename into 3 chunks so we can color output
                    let file_name_pre_match = file_name_str.split_at(m.start()).0;
                    let file_name_match = m.as_str();
                    let file_name_post_match = file_name_str.split_at(m.end()).1;

                    // Print out the parent path, relative to `root_dir`
                    let mut parent_path = f.parent().unwrap().to_str().unwrap();
                    parent_path = parent_path.trim_left_matches(root_dir.to_str().unwrap());

                    (write!(stdout, "{}{}", parent_path, path::MAIN_SEPARATOR)).unwrap();
                    (write!(stdout, "{}", file_name_pre_match)).unwrap();

                    stdout.fg(term::color::GREEN).unwrap();
                    (write!(stdout, "{}", file_name_match)).unwrap();
                    stdout.reset().unwrap();

                    (writeln!(stdout, "{}", file_name_post_match)).unwrap();
                }
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

/// Spawns a simple progress bar that ticks until a message
/// is passed to `reciever`
fn show_progress<T>(reciever: Receiver<T>) {
    let mut counter = 0;
    let counter_prefix = "Searching...";

    loop {
        match counter {
            0 => print!("\r{}|", counter_prefix),
            1 => print!("\r{}/", counter_prefix),
            2 => print!("\r{}-", counter_prefix),
            3 => print!("\r{}\\", counter_prefix),
            _ => print!(".")
        };
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(250));

        if counter == 3 {
            counter = 0;
        } else {
            counter += 1;
        }

        match reciever.try_recv() {
            Ok(_) => {
                print!("\r");
                io::stdout().flush().unwrap();
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {},
            Err(_) => break
        };
    }
}
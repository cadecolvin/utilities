//! Prepend is a tool used to insert a line to the beginning of a file
//!
//! Running `prepend test temp.txt` is the same as running `sed "i1test" temp.txt'

use std::io::{BufRead, BufReader, BufWriter};
use std::io::{Read, Write};
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::process;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() {
    let matches = App::new("Prepend")
                        .version(crate_version!())
                        .author("Cade Colvin <cade.colvin@gmail.com>")
                        .about("Prpends `TEXT` to the beginning of `FILE`")
                        .arg(Arg::with_name("text")
                            .index(1)
                            .required(true)
                            .help("The text to prepend"))
                        .arg(Arg::with_name("file")
                            .index(2)
                            .required(true)
                            .help("The file to prepend `TEXT` to"))
                        .get_matches();

    let file_path_arg = matches.value_of("file").unwrap();
    let prepend_text = matches.value_of("text").unwrap();

    let mut file_path = PathBuf::from(file_path_arg);
    let original_extension = match file_path.extension() {
        Some(e) => e.to_str().unwrap(),
        None => "",
    };
    
    let mut backup_file_path = PathBuf::from(file_path_arg);
    backup_file_path.set_extension(original_extension.to_owned() + ".bak");

    // Move the original file to the backup path
    match fs::rename(file_path.as_path(), backup_file_path.as_path()) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Unable to create .bak file");
            process::exit(1);
        }
    };

    let mut file_reader = match File::open(backup_file_path) {
        Ok(f) => BufReader::new(f),
        Err(_) => {
            eprintln!("Unable to open FILE.bak for reading");
            process::exit(1);
        }
    };

    let mut file_writer = match File::create(file_path.as_path()) {
        Ok(f) => BufWriter::new(f),
        Err(_) => {
            eprintln!("Unable to create FILE for writing");
            process::exit(1);
        }
    };

    // Write our data to the new file
    writeln!(file_writer, "{}", prepend_text);
    for line in file_reader.lines() {
        writeln!(file_writer, "{}", line.unwrap());
    }
}

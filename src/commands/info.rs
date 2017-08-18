
use clap::ArgMatches;
use time::strptime;

use std::io::BufReader;
use std::fs::File;

use super::super::types::*;
use super::super::parser;

pub fn execute(matches: &ArgMatches) {
    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);
    match parser::read_header(&mut reader) {
        Ok(data) => {
            let header: DDMainHeader = data.0;
            let files: Vec<DDSubFileHeader> = data.1;
            if matches.is_present("verbose") {
                println!("## HEADER");
                println!("magic bytes: {:?}", header.magic_number);
                println!("header length: {}", header.header_length);
                println!("## FILES");
            } else {
                println!("{} file{}", files.len(), if files.len() == 1 {""} else {"s"});
            }

            for file in files {
                if matches.is_present("verbose") {
                    println!("offset {} B\tdatetime {}\t{} B\t{}\t{:?}",
                             file.offset,
                             strptime(format!("{}", file.timestamp).as_mut_str(), "%s").unwrap().rfc3339(),
                             file.size,
                             file.filename,
                             file.file_type
                    );
                } else if matches.is_present("types") {
                    println!("{}{}: {} MB, {}",
                             file.filename,
                             if matches.is_present("extensions") {file.file_type.extension()} else {"".to_string()},
                             (file.size as f32)/1024f32/1024f32,
                             file.file_type
                    );
                } else {
                    println!("{}: {} MB",
                             file.filename,
                             (file.size as f32)/1024f32/1024f32,
                    );
                }
            }
        },
        Err(err) => {
            println!("Error! {:?}", err);
            return;
        }
    }
}
#[macro_use]
extern crate nom;
extern crate time;

mod parser;
use parser::*;

use nom::IResult::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn main() {
    let f = File::open("audio").unwrap();
    let mut reader = BufReader::new(f);

    let mut buf: Vec<u8> = vec![];

    reader.read_to_end(&mut buf);

    println!("bytes: {}", buf.len());
    //println!("stuff: {:?}", buf);

    match header_section_bound(buf.as_ref()) {
        Done(unparsed, parsed) => {
            println!("parsed!");
            let header: DDMainHeader = parsed.0;
            let files: Vec<DDSubFileHeader> = parsed.1;
            println!("## HEADER");
            println!("magic bytes: {:?}", header.magic_number);
            println!("header length: {}", header.header_length);
            println!("## FILES");
            for file in files {
                println!("offset {}b\tdatetime {}\t{} MB ({})\t{}",
                         file.offset,
                         time::strptime(format!("{}", file.timestamp).as_mut_str(), "%s").unwrap().rfc3339(),
                         (file.size as f32)/1024f32/1024f32,
                         file.size,
                         file.filename
                );
            }
        },
        Error(err) => {
            println!("error: {:?}", err);
            println!("{}", err.description());
        },
        Incomplete(needed) => {
            println!("need {:?} more bytes", needed);
        }
    }
}

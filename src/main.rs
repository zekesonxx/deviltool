#[macro_use]
extern crate nom;
extern crate time;

pub const DD_HEADER_LENGTH: usize = 12;

#[derive(Debug, PartialEq)]
struct DDMainHeader {
    /// The magic number at the start of the file
    /// Should always be `:hx:rg:\01`
    magic_number: Vec<u8>, // should be [u8; 8]
    /// Length of the header
    /// You only turn this into an offset if you add 12 to it, which the original C code was doing.
    header_length: u32
}

#[derive(Debug, PartialEq)]
struct DDSubFileHeader {
    /// File type
    file_type: u16, // seems to be 0x20 for audio, and 0x10/0x11 and others for textures (dd), and 0x00 for the end of the header lump (for the first invalid fileheader)
    /// Filename
    filename: String,
    /// File's position (offset in bytes from the beginning of the file)
    offset: u32,
    /// Length/size of the file, in bytes
    size: u32,
    /// Unix timestamp.
    /// File creation/modification times?
    timestamp: u32
}

use nom::IResult;
use nom::IResult::*;
use nom::Needed;
use nom::{le_u16, le_u32};

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

named!(mainheader<DDMainHeader>,
    do_parse!(
        magic: take!(8) >>
        offset: le_u32 >>
        (DDMainHeader {
            magic_number: Vec::from(magic),
            header_length: offset //+ DD_HEADER_LENGTH as u32
        })
    )
);

named!(subheader<DDSubFileHeader>,
    do_parse!(
        file_type: le_u16 >>
        filename: take_until_and_consume_s!("\0") >>
        offset: le_u32 >>
        size: le_u32 >>
        timestamp: le_u32 >>
        (DDSubFileHeader {
            file_type: file_type,
            filename: String::from_utf8_lossy(filename).into_owned(),
            offset: offset,
            size: size,
            timestamp: timestamp
        })
    )
);

named!(header_section_bound<(DDMainHeader, Vec<DDSubFileHeader>)>,
    do_parse!(
        main: mainheader >>
        files: flat_map!(take!(main.header_length), many_till!(call!(subheader), tag!("\0\0"))) >>
        (main, files.0)
    )
);


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

// 632092 B	offset 7241b	datetime 2016-04-14T11:11:00Z	andrasimpact

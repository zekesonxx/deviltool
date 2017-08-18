use clap::ArgMatches;
use nom::IResult::*;
use image::{self, GenericImage, ImageBuffer};

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use super::super::tex2;

pub fn execute(matches: &ArgMatches) {
    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::with_capacity(5000);
    reader.read_to_end(&mut buf);
    match tex2::tex2_image_boundless(buf.as_ref()) {
        Done(unused, tex2image) => {
            let totalpixels = (tex2image.width*tex2image.height) as usize;
            let extrapixels= tex2image.pixels.len() - totalpixels;
            print!("{}: ", matches.value_of("FILE").unwrap());
            print!("{height}x{width} ({totalpixels}), unknown {unknown:#X}, extra pixels: {extra}",
                     height=tex2image.height,
                     width=tex2image.width,
                     totalpixels=totalpixels,
                     unknown=tex2image.unknown1,
                     extra=extrapixels
            );
            println!(", unused: {}", unused.len());

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
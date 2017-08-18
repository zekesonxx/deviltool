
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
        Done(unused, mut tex2image) => {
            println!("parsed!");

            let mut img = ImageBuffer::new(tex2image.width, tex2image.height);
            if matches.is_present("phantom") {
                tex2image.phantom = true;
            }
            img.copy_from(&tex2image, 0, 0);

            let mut output_file = PathBuf::from(matches.value_of("FILE").unwrap());
            output_file.set_extension("png");

            let ref mut fout = File::create(output_file).unwrap();

            // We must indicate the image’s color type and what format to save as
            let _ = image::ImageRgba8(img).save(fout, image::PNG);
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

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
            let mut output_file = if matches.is_present("OUTFILE") {
                PathBuf::from(matches.value_of("OUTFILE").unwrap())
            } else {
                let mut file = PathBuf::from(matches.value_of("FILE").unwrap());
                file.set_extension("png");
                file
            };

            let mut img = ImageBuffer::new(tex2image.width, tex2image.height);
            img.copy_from(&tex2image, 0, 0);

            let ref mut fout = File::create(output_file.clone()).unwrap();

            let _ = image::ImageRgba8(img).save(fout, image::PNG);

            println!("Converted image saved to {}", output_file.display());
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
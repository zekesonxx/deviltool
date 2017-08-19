
use clap::ArgMatches;
use nom::IResult::*;
use image::{self, GenericImage, ImageBuffer};

use std::io::prelude::*;
use std::io::{self, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;

use super::super::tex2;

pub fn execute(matches: &ArgMatches) {
    let tex2image = match read_tex2(matches.value_of("FILE").unwrap()) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to read file {}", matches.value_of("FILE").unwrap());
            println!("{:?}", e);
            exit(1);
        }
    };

    let mut output_file = if matches.is_present("OUTFILE") {
        PathBuf::from(matches.value_of("OUTFILE").unwrap())
    } else {
        let mut file = PathBuf::from(matches.value_of("FILE").unwrap());
        file.set_extension("png");
        file
    };

    match save_to_png(output_file.clone(), tex2image) {
        Ok(_) => {
            println!("Converted image saved to {}", output_file.display());
        },
        Err(e) => {
            println!("Error saving image to file {}", output_file.display());
            println!("{:?}", e);
            exit(1);
        }
    }
}

pub fn read_tex2<P: AsRef<Path>>(file: P) -> io::Result<tex2::DDTex2Image> {
    let f = File::open(file)?;
    let mut reader = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::with_capacity(5000);
    reader.read_to_end(&mut buf)?;
    match tex2::tex2_image_boundless(buf.as_ref()) {
        Done(_, tex2image) => {
            return Ok(tex2image);
        },
        Error(err) => {
            println!("error: {:?}", err);
            println!("{}", err.description());
            exit(1);
        },
        Incomplete(needed) => {
            println!("need {:?} more bytes", needed);
            exit(1);
        }
    }
}

pub fn save_to_png<P: AsRef<Path>>(output_file: P, tex2img: tex2::DDTex2Image) -> io::Result<()> {
    let mut img = ImageBuffer::new(tex2img.width, tex2img.height);
    img.copy_from(&tex2img, 0, 0);

    let ref mut fout = File::create(output_file)?;
    image::ImageRgba8(img).save(fout, image::PNG);
    Ok(())
}
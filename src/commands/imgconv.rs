
use clap::ArgMatches;
use nom::IResult;
use image::{self, GenericImage, ImageBuffer};

use std::io::prelude::*;
use std::io::{self, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;

use super::super::tex2;
use super::super::errors::*;

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let mut tex2image = read_tex2(matches.value_of("FILE").unwrap()).chain_err(|| "Failed to open input file")?;

    let mut output_file = if matches.is_present("OUTFILE") {
        PathBuf::from(matches.value_of("OUTFILE").unwrap())
    } else {
        let mut file = PathBuf::from(matches.value_of("FILE").unwrap());
        file.set_extension("png");
        file
    };

    let max_mipmap_levels = if matches.is_present("mipmaps") { tex2image.mipmap_levels } else { 1 } as usize;

    let mut _output_file = output_file.clone();
    _output_file.set_extension("");
    let filename = _output_file.file_name().unwrap_or("converted_dd_tex2".as_ref()).to_str().unwrap();
    let ext = _output_file.extension().unwrap_or("png".as_ref()).to_str().unwrap();

    for i in 0..(max_mipmap_levels) {
        tex2image.set_mipmap(i as u8);
        if i != 0 {
            output_file.set_file_name(format!("{}_{}.{}", filename, i, ext));
        }
        match save_to_png(output_file.clone(), &tex2image) {
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
    Ok(())
}

pub fn read_tex2<P: AsRef<Path>>(file: P) -> io::Result<tex2::DDTex2Image> {
    let f = File::open(file)?;
    let mut reader = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::with_capacity(5000);
    reader.read_to_end(&mut buf)?;
    match tex2::tex2_image(buf.as_ref()) {
        IResult::Done(_, tex2image) => {
            return Ok(tex2image);
        },
        IResult::Error(err) => {
            println!("Failed to read tex2 image");
            println!("{}", err.description());
            exit(1);
        },
        IResult::Incomplete(needed) => {
            println!("need {:?} more bytes", needed);
            exit(1);
        }
    }
}

pub fn save_to_png<P: AsRef<Path>>(output_file: P, tex2img: &tex2::DDTex2Image) -> Result<()> {
    let mut img = ImageBuffer::new(tex2img.cur_width(), tex2img.cur_height());
    img.copy_from(tex2img, 0, 0);

    let ref mut fout = File::create(output_file).chain_err(|| "Failed to open output image file")?;
    image::ImageRgba8(img).save(fout, image::PNG).chain_err(|| "Failed to save output image")?;
    Ok(())
}
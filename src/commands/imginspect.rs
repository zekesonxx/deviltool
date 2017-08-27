use clap::ArgMatches;
use nom::IResult::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use super::super::tex2;

pub fn execute(matches: &ArgMatches) {
    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);
    let mut buf: Vec<u8> = Vec::with_capacity(5000);
    reader.read_to_end(&mut buf);
    match tex2::tex2_image(buf.as_ref()) {
        Done(unused, tex2image) => {
            let totalpixels = (tex2image.width*tex2image.height) as usize;
            let extrapixels= tex2image.pixels.len() - totalpixels;
            print!("{}: ", matches.value_of("FILE").unwrap());
            print!("{width}x{height} ({totalpixels}), unknown {unknown:#X}, extra pixels: {extra}",
                     height=tex2image.height,
                     width=tex2image.width,
                     totalpixels=totalpixels,
                     unknown=tex2image.mipmap_levels,
                     extra=extrapixels
            );
            println!(", unused: {}", unused.len());
            let widthexp = (tex2image.width as f32).log(2f32) as u32;
            let heightexp = (tex2image.height as f32).log(2f32) as u32;

            let mut remainder = extrapixels;
            for i in 1..(tex2image.mipmap_levels) {
                let width = 2u32.pow(widthexp - i as u32);
                let height = 2u32.pow(heightexp - i as u32);
                remainder -= (width*height) as usize;
                println!("- {}x{}: {} pixels, remaining: {}",
                    width,
                    height,
                    width*height,
                    remainder
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
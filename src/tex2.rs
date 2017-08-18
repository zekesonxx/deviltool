
use std::io::{self, Write};

use image;
use image::{ImageBuffer, GenericImage, Rgba};
use nom::{IError, le_u8, le_u16, le_u32};
use byteorder::{LittleEndian, WriteBytesExt};

named!(pub tex2_header<(u32, u32, u8)>,
    do_parse!(
        tag!("\x11\x40") >> //.@, the magic number for the format
        height: le_u32 >>
        width: le_u32 >>
        unknown: le_u8 >> // unknown
        (height, width, unknown)
    )
);

named!(pub tex2_pixel<(u8, u8, u8, u8)>,
    tuple!(
        le_u8,
        le_u8,
        le_u8,
        le_u8
    )
);

named!(pub tex2_image<DDTex2Image>,
    do_parse!(
        header: tex2_header >>
        pixels: count!(tex2_pixel, (header.0*header.1) as usize) >>
        (DDTex2Image {
            height: header.0,
            width: header.1,
            pixels: pixels
        })
    )

);

pub struct DDTex2Image {
    pub height: u32,
    pub width: u32,
    pixels: Vec<(u8, u8, u8, u8)>
}

impl DDTex2Image {
    pub fn new(width: u32, height: u32) -> Self {
        DDTex2Image {
            height: height,
            width: width,
            pixels: vec![(0, 0, 0, 0); (height*width) as usize]
        }
    }

    pub fn save(&self, mut dst: &mut Write) -> io::Result<()> {
        dst.write_u8(0x11)?;
        dst.write_u8(0x40)?;
        dst.write_u32::<LittleEndian>(self.height)?;
        dst.write_u32::<LittleEndian>(self.width)?;
        dst.write_u8(0x08)?; //no idea what this is
        for pixel in self.pixels.iter() {
            dst.write_u8(pixel.0)?;
            dst.write_u8(pixel.1)?;
            dst.write_u8(pixel.2)?;
            dst.write_u8(pixel.3)?;
        }
        Ok(())
    }
}

impl GenericImage for DDTex2Image {
    type Pixel = image::Rgba<u8>;

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width, self.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let p = self.pixels[((self.width*y) + x) as usize];
        image::Rgba([p.0, p.1, p.2, p.3])
    }

    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Self::Pixel {
        unimplemented!()
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.pixels[((self.width*y) + x) as usize] = (pixel[0], pixel[1], pixel[2], pixel[3])
    }

    fn blend_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        unimplemented!()
    }
}
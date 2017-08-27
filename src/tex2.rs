
use std::io::{self, Write};

use image;
use image::GenericImage;
use nom::{le_u8, le_u32};
use byteorder::{LittleEndian, WriteBytesExt};

named!(pub tex2_header<(u32, u32, u8)>,
    do_parse!(
        tag!("\x11\x40") >> //.@, the magic number for the format
        height: le_u32 >>
        width: le_u32 >>
        mipmaps: le_u8 >>
        (height, width, mipmaps)
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
        pixels: count!(tex2_pixel, calc_offset(header.0, header.1, (header.2 as u32)+1) as usize) >>
        (DDTex2Image {
            mipmap_levels: header.2,
            mipmap_current: 0,
            height: header.0,
            width: header.1,
            pixels: pixels
        })
    )
);

pub struct DDTex2Image {
    pub mipmap_levels: u8,
    mipmap_current: u8,
    pub height: u32,
    pub width: u32,
    pub pixels: Vec<(u8, u8, u8, u8)>
}

impl DDTex2Image {
    pub fn new(width: u32, height: u32) -> Self {
        DDTex2Image {
            mipmap_levels: 0,
            mipmap_current: 0,
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
        dst.write_u8(self.mipmap_levels)?;
        for pixel in self.pixels.iter() {
            dst.write_u8(pixel.0)?;
            dst.write_u8(pixel.1)?;
            dst.write_u8(pixel.2)?;
            dst.write_u8(pixel.3)?;
        }
        Ok(())
    }

    pub fn set_mipmap(&mut self, new: u8) {
        if new != 0 && (!self.height.is_power_of_two() || !self.width.is_power_of_two()) {
            panic!("DDTex2Image {}x{}: can't do mipmap levels on non-power-of-two!", self.width, self.height);
        }
        self.mipmap_current = new;
    }

    pub fn cur_width(&self) -> u32 {
        self.width >> self.mipmap_current
    }

    pub fn cur_height(&self) -> u32 {
        self.height >> self.mipmap_current
    }

    pub fn pos(&self, x: u32, y: u32) -> usize {
        (calc_offset(self.height, self.width, (self.mipmap_current) as u32)
            + ((self.cur_width()*y) + x)) as usize
    }
    pub fn pixel(&self, x: u32, y: u32) -> (u8, u8, u8, u8) {
        match self.pixels.get(self.pos(x, y)) {
            Some(p) => *p,
            None => (0xFF, 0xFF, 0xFF, 0xFF)
        }
    }
}

#[allow(unused_variables)]
impl GenericImage for DDTex2Image {
    type Pixel = image::Rgba<u8>;

    fn dimensions(&self) -> (u32, u32) {
        (self.cur_width(), self.cur_height())
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.cur_width(), self.cur_height())
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let p = self.pixel(x, y);
        image::Rgba([p.0, p.1, p.2, p.3])
    }

    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Self::Pixel {
        unimplemented!()
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        let pos = self.pos(x, y);
        self.pixels[pos] = (pixel[0], pixel[1], pixel[2], pixel[3]);
    }

    fn blend_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        unimplemented!()
    }
}


//12:36:48 AM <ubsan> zekesonxx: lemme think about it
//12:36:51 AM <ubsan> I have an idea
//12:37:24 AM <zekesonxx> ubsan: shoot
//12:37:42 AM <ubsan> zekesonxx: alright, so the function definition would be something like
//12:37:57 AM <ubsan> fn foo(x: usize, y: usize, n: u32) -> usize {
//12:39:51 AM <ubsan>   let (x, y) = (x.trailing_zeroes(), y.trailing_zeroes()); (0..n).fold(1, |acc, n| acc += 1 << (x - n) * 1 << (y - n))
//12:39:55 AM <ubsan> something like this ^?
//12:52:55 AM <zekesonxx> ubsan: little bit of reworking needed but looks good. Thanks for the help.
//12:53:01 AM <ubsan> zekesonxx: yep!
//12:53:10 AM * ubsan has done some assembly in the past :P
fn calc_offset(height: u32, width: u32, cur_mip: u32) -> u32 {
    let (x, y) = (height.trailing_zeros(), width.trailing_zeros());
    (0..cur_mip).fold(0u32, |acc, n| acc + (1 << (x - n) * 1 << (y - n)))
}
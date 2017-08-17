
use image;
use image::{ImageBuffer, GenericImage, Rgba};
use nom::{IError, le_u8, le_u16, le_u32};

named!(pub tex2_header<(u32, u32)>,
    do_parse!(
        tag!("\x11\x40") >> //.@, the magic number for the format
        width: le_u32 >>
        height: le_u32 >>
        le_u8 >> // unknown
        (width, height)
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

named!(pub tex2_image<((u32, u32), Vec<(u8, u8, u8, u8)>)>,
    do_parse!(
        header: tex2_header >>
        pixel: count!(tex2_pixel, (header.0*header.1) as usize) >>
        (header, pixel)
    )

);


use std::io::prelude::*;
use std::io::{self, Write, BufReader};
use nom::{IError, le_u16, le_u32};
use nom::IResult::*;
use nom::Needed::Size;

use types::*;


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
            file_type: DDFiletype::new(file_type),
            filename: String::from_utf8_lossy(filename).into_owned(),
            offset: offset,
            size: size,
            timestamp: timestamp
        })
    )
);

named!(pub header_section_bound<(DDMainHeader, Vec<DDSubFileHeader>)>,
    do_parse!(
        main: mainheader >>
        files: flat_map!(take!(main.header_length), many_till!(call!(subheader), tag!("\0\0"))) >>
        (main, files.0)
    )
);

named!(pub glsl_file<(String, String, String)>,
    do_parse!(
        name_len: le_u32 >>
        vert_len: le_u32 >>
        frag_len: le_u32 >>
        name: take_str!(name_len) >>
        vert: take_str!(vert_len) >>
        frag: take_str!(frag_len) >>
        (name.to_string(), vert.to_string(), frag.to_string())
    )
);


pub fn read_header<R: Read>(reader: &mut R) -> Result<(DDMainHeader, Vec<DDSubFileHeader>), IError> {
    let mut header: Vec<u8> = vec![0; 12];
    reader.read_exact(&mut header[..12]);
    match header_section_bound(header.as_ref()) {
        Incomplete(Size(size)) => {
            header.append(&mut vec![0; size]);
            reader.read_exact(&mut header[12..size+12]);
            header_section_bound(header.as_ref()).to_full_result()
        },
        _ => {
            use nom::Needed::Unknown;
            return Err(IError::Incomplete(Unknown));
        }
    }
}

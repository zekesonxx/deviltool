use nom::{le_u16, le_u32};

#[derive(Debug, PartialEq)]
pub struct DDMainHeader {
    /// The magic number at the start of the file
    /// Should always be `:hx:rg:\01`
    pub magic_number: Vec<u8>, // should be [u8; 8]
    /// Length of the header
    /// You only turn this into an offset if you add 12 to it, which the original C code was doing.
    pub header_length: u32
}

#[derive(Debug, PartialEq)]
pub struct DDSubFileHeader {
    /// File type
    pub file_type: u16, // seems to be 0x20 for audio, and 0x10/0x11 and others for textures (dd), and 0x00 for the end of the header lump (for the first invalid fileheader)
    /// Filename
    pub filename: String,
    /// File's position (offset in bytes from the beginning of the file)
    pub offset: u32,
    /// Length/size of the file, in bytes
    pub size: u32,
    /// Unix timestamp.
    /// File creation/modification times?
    pub timestamp: u32
}

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
            file_type: file_type,
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

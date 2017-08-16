
use std::fmt;
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
    pub file_type: DDFiletype,
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

// seems to be 0x20 for audio, and 0x10/0x11 and others for textures (dd), and 0x00 for the end of the header lump (for the first invalid fileheader)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DDFiletype {
    /// 0x20, little-endian WAVE audio, 44100 Hz 16 bit PCM
    ///
    /// `file` output:
    ///
    /// audio/andrasimpact.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, stereo 44100 Hz
    WavAudio,
    Unknown(u16)
}

impl DDFiletype {
    pub fn new(input: u16) -> Self {
        use DDFiletype::*;
        match input {
            0x20 => WavAudio,
            _ => Unknown(input)
        }
    }
}

impl fmt::Display for DDFiletype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DDFiletype::*;
        match *self {
            WavAudio => write!(f, "wav audio"),
            Unknown(t) => write!(f, "unknown ({:#X})", t)
        }
    }
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

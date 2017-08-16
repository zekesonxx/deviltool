
use std::fmt;
use std::io::prelude::*;
use std::io::BufReader;
use nom::{IError, le_u16, le_u32};
use nom::IResult::*;
use nom::Needed::Size;

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
    /// 0x80, shader text file of some sort
    ShaderText,
    /// 0x10, GLSL with some bytes at the beginning
    WeirdGLSL,
    /// 0x01, Some sort of texture data
    Texture1,
    /// 0x02, Some sort of texture data
    Texture2,
    /// 0x11, Folder Marker. Probably.
    FolderMarker,
    Unknown(u16)
}

impl DDFiletype {
    pub fn new(input: u16) -> Self {
        use DDFiletype::*;
        match input {
            0x20 => WavAudio,
            0x80 => ShaderText,
            0x10 => WeirdGLSL,
            0x01 => Texture1,
            0x02 => Texture2,
            0x11 => FolderMarker,
            _ => Unknown(input)
        }
    }

    pub fn extension(&self) -> String {
        use DDFiletype::*;
        match *self {
            WavAudio => "wav".to_string(),
            ShaderText => "shadercfg".to_string(),
            WeirdGLSL => "dd_glsl".to_string(),
            Texture1 => "dd_tex1".to_string(),
            Texture2 => "dd_tex2".to_string(),
            FolderMarker => "foldermarker".to_string(),
            Unknown(t) => format!("dd_{:#X}", t)
        }
    }
}

impl fmt::Display for DDFiletype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DDFiletype::*;
        match *self {
            WavAudio => write!(f, "wav audio"),
            ShaderText => write!(f, "shader text file"),
            WeirdGLSL => write!(f, "glsl shader"),
            Texture1 => write!(f, "texture or something 1"),
            Texture2 => write!(f, "texture or something 2"),
            FolderMarker => write!(f, "folder marker probably"),
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

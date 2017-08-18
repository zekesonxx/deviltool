
use std::fmt;
use std::io::prelude::*;
use std::io::{self, Write, BufReader};
use byteorder::{LittleEndian, WriteBytesExt};
use nom::{IError, le_u16, le_u32};
use nom::IResult::*;
use nom::Needed::Size;

#[derive(Debug, PartialEq)]
pub struct DDMainHeader {
    /// The magic number at the start of the file.
    ///
    /// Should always be `:hx:rg:\01`
    pub magic_number: Vec<u8>, // should be [u8; 8]
    /// Length of the header.
    ///
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

impl DDSubFileHeader {
    pub fn write(&self, mut dst: &mut Write) -> io::Result<()> {
        dst.write_u16::<LittleEndian>(self.file_type.to_u16())?;
        dst.write(self.filename.as_bytes())?;
        dst.write_u8(0)?; // Null term for filename
        dst.write_u32::<LittleEndian>(self.offset)?;
        dst.write_u32::<LittleEndian>(self.size)?;
        dst.write_u32::<LittleEndian>(self.timestamp)?;
        Ok(())
    }
}


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
    /// 0x10, Combined GLSL vertex and fragment shader.
    GLSL,
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
            0x10 => GLSL,
            0x01 => Texture1,
            0x02 => Texture2,
            0x11 => FolderMarker,
            _ => Unknown(input)
        }
    }
    pub fn to_u16(&self) -> u16 {
        use DDFiletype::*;
        match *self {
            WavAudio => 0x20,
            ShaderText => 0x80,
            GLSL => 0x10,
            Texture1 => 0x01,
            Texture2 => 0x02,
            FolderMarker => 0x11,
            Unknown(t) => t
        }
    }

    pub fn extension(&self) -> String {
        use DDFiletype::*;
        match *self {
            WavAudio => "wav".to_string(),
            ShaderText => "shadercfg".to_string(),
            GLSL => "dd_glsl".to_string(),
            Texture1 => "dd_tex1".to_string(),
            Texture2 => "dd_tex2".to_string(),
            FolderMarker => "foldermarker".to_string(),
            Unknown(t) => format!("dd_{:#X}", t)
        }
    }
    pub fn from_extension(ext: &str) -> Option<Self> {
        use DDFiletype::*;
        match ext {
            "wav" => Some(WavAudio),
            "dd_glsl" => Some(GLSL),
            "dd_tex1" => Some(Texture1),
            "dd_tex2" => Some(Texture2),
            "shadercfg" => Some(ShaderText),
            _ => None
        }
    }
}

impl fmt::Display for DDFiletype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DDFiletype::*;
        match *self {
            WavAudio => write!(f, "wav audio"),
            ShaderText => write!(f, "shader text file"),
            GLSL => write!(f, "glsl vert+frag shader"),
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

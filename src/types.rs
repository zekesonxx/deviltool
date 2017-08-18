
use std::fmt;

use std::io::prelude::*;
use std::io;
use byteorder::{LittleEndian, WriteBytesExt};

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
        use self::DDFiletype::*;
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
        use self::DDFiletype::*;
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
        use self::DDFiletype::*;
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
        use self::DDFiletype::*;
        match ext {
            "wav" => Some(WavAudio),
            "dd_glsl" => Some(GLSL),
            "dd_tex1" => Some(Texture1),
            "dd_tex2" => Some(Texture2),
            "shadercfg" => Some(ShaderText),
            _ => None
        }
    }
    pub fn is_unknown(&self) -> bool {
        use self::DDFiletype::*;
        if let &Unknown(_) = self {
            true
        } else {
            false
        }
    }
}

impl fmt::Display for DDFiletype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DDFiletype::*;
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
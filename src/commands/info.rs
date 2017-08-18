
use clap::ArgMatches;
use time::strptime;

use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use nom::IResult;
use bytesize::ByteSize;

use super::super::types::*;
use super::super::parser;
use super::super::tex2;

#[derive(Debug, PartialEq)]
enum GuessedFormat {
    DDArchive,
    Texture2,
    GLSLShader,
    Unknown
}

pub fn execute(matches: &ArgMatches) {
    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);

    let format = match guess_format(&mut reader) {
        Ok(format) => format,
        Err(e) => {
            println!("Failed to read file somehow!");
            println!("{:?}", e);
            return;
        }
    };

    use self::GuessedFormat::*;
    match format {
        DDArchive => {
            archive_info(matches, &mut reader);
        },
        Texture2 => {
            texture_info(matches, &mut reader);
        },
        GLSLShader => {
            glsl_info(matches, &mut reader);
        },
        _ => {
            println!("{}: unknown", matches.value_of("FILE").unwrap());
        }
    }
}

fn guess_format<R: Read + Seek>(mut reader: &mut R) -> io::Result<GuessedFormat> {
    use self::GuessedFormat::*;

    // Read out the first 40 bytes of the file
    // And use that to take guesses at it
    let mut buf = vec![0u8; 40];
    reader.read_exact(&mut buf[..])?;
    // restart the position for whatever wants to read this next
    reader.seek(SeekFrom::Start(0));

    // Start with an archive
    if let IResult::Done(_, _) = parser::mainheader(&buf) {
        return Ok(DDArchive);
    }

    // Texture2 file?
    if let IResult::Done(_, _) = tex2::tex2_header(&buf) {
        return Ok(Texture2);
    }

    // GLSL shader file?
    if let IResult::Done(_, _) = parser::glsl_file_header(&buf) {
        return Ok(GLSLShader);
    }


    Ok(Unknown)
}

fn archive_info<R: Read>(matches: &ArgMatches, mut reader: &mut R) -> io::Result<()> {
    if let Ok((header, files)) = parser::read_header(&mut reader)? {
        let totalsize = files.iter().fold(0, |acc, ref x| acc + x.size) as usize;
        println!("{filename}: dd archive, {header} byte header, {count} file{countplural}, totaling {total}",
                 filename=matches.value_of("FILE").unwrap(),
                 count=files.len(),
                 countplural=if files.len() == 1 {""} else {"s"},
                 header=header.header_length + 12,
                 total=ByteSize::b(totalsize).to_string(true)
        );

        if matches.is_present("list") || matches.is_present("dump") {
            for file in files {
                if matches.is_present("dump") {
                    println!("offset {offset:9} | datetime {datetime} | size {size:9} | {ftype:>11} | {name}",
                             offset=file.offset,
                             datetime=strptime(format!("{}", file.timestamp).as_mut_str(), "%s").unwrap().rfc3339(),
                             size=file.size,
                             name=file.filename,
                             ftype=format!("{:?}", file.file_type)
                    );
                } else {
                    println!("- {filename}{ext}: {ftype}, {size}{offset}",
                             filename = file.filename,
                             size = ByteSize::b(file.size as usize),
                             ftype = file.file_type,
                             offset = if matches.is_present("offset") {
                                 format!(", offset {}", file.offset)
                             } else {
                                 "".to_string()
                             },
                             ext = if matches.is_present("extensions") {
                                 format!(".{}", file.file_type.extension())
                             } else {
                                 "".to_string()
                             }
                    );
                }
            }
        }
    } else {
        println!("A very strange error occurred");
    }
    Ok(())
}

fn texture_info<R: Read>(matches: &ArgMatches, mut reader: &mut R) -> io::Result<()> {
    let mut buf = vec![0u8; 11];
    reader.read_exact(&mut buf[..])?;
    if let IResult::Done(_, info) = tex2::tex2_header(&buf) {
        println!("{}: texture2, {}x{}, unknown byte {:#X}",
            matches.value_of("FILE").unwrap(),
            info.0,
            info.1,
            info.2
        );
    } else {
        println!("A very strange error occurred");
    }
    Ok(())
}

fn glsl_info<R: Read>(matches: &ArgMatches, mut reader: &mut R) -> io::Result<()> {
    let mut buf = vec![];
    reader.read_to_end(&mut buf)?;
    if let IResult::Done(_, info) = parser::glsl_file(&buf) {
        println!("{}: glsl vert+frag shader \"{}\"\nvertex shader: {} lines ({} bytes)\nfragment shader: {} lines ({} bytes)",
                 matches.value_of("FILE").unwrap(),
                 info.0,
                 info.1.lines().count(),
                 info.1.len(),
                 info.2.lines().count(),
                 info.2.len()
        );
    } else {
        println!("A very strange error occurred");
    }
    Ok(())
}
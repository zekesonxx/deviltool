
use clap::ArgMatches;

use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::fs::{self, File};
use std::path::PathBuf;
use nom::IResult;
use filetime::{self, FileTime};

use super::super::types::*;
use super::super::parser;
use super::super::errors::*;

pub fn execute(matches: &ArgMatches) -> Result<()> {
    // make sure we have somewhere to put the files
    let mut output_dir = PathBuf::from(matches.value_of("FOLDER").unwrap());
    fs::create_dir_all(output_dir.clone()).chain_err(|| "Failed to create output directory")?;

    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);
    let mut firstfolder = true;
    let (_, mut files) = parser::read_header(&mut reader).unwrap().unwrap(); //TODO this

    if !matches.is_present("nofolders") { files.reverse(); }
    for file in files {
        if file.file_type == DDFiletype::FolderMarker {
            if matches.is_present("foldermarkers") {
                // do nothing
            } else if matches.is_present("nofolders") {
                println!("Ignoring folder marker {}", file.filename);
                continue;
            } else {
                if !firstfolder {
                    output_dir.pop();
                } else {
                    firstfolder = false;
                }
                output_dir.push(file.filename.clone());
                fs::create_dir_all(output_dir.clone())
                    .chain_err(|| "Failed to create subdirectory")?;
                continue;
            }
        }
        let mut output_file = output_dir.join(file.filename.clone());
        output_file.set_extension(file.file_type.extension());

        {
            reader.seek(SeekFrom::Start(file.offset as u64))
                .chain_err(|| "Failed to seek to a position within archive")?;
            let mut buf = vec![0; file.size as usize];
            reader.read_exact(&mut buf[..])
                .chain_err(|| "Failed to read file from archive")?;

            if file.file_type == DDFiletype::GLSL && !matches.is_present("preserveglsl") {
                match parser::glsl_file(buf.as_ref()) {
                    IResult::Incomplete(_) | IResult::Error(_) => {
                        println!("Malformed GLSL file! Saving as normal file");
                    },
                    IResult::Done(_, (name, vertex, fragment)) => {
                        if name != file.filename {
                            println!("Warning: GLSL name is {} but saving as {}",
                                     name, file.filename);
                        }
                        output_file.set_extension("vert");
                        println!("Writing {}", output_file.display());
                        let mut file_handle = File::create(output_file.clone())
                            .chain_err(|| "Failed to open GLSL vertex shader file")?;
                        file_handle.write_all(vertex.as_bytes())
                            .chain_err(|| "Failed to save GLSL vertex shader file")?;

                        output_file.set_extension("frag");
                        println!("Writing {}", output_file.display());
                        let mut file_handle = File::create(output_file.clone())
                            .chain_err(|| "Failed to open GLSL fragment shader file")?;
                        file_handle.write_all(fragment.as_bytes())
                            .chain_err(|| "Failed to save GLSL fragment shader file")?;
                        continue;
                    }
                }
            }

            println!("Writing {}", output_file.display());
            let mut file_handle = File::create(output_file.clone())
                .chain_err(|| format!("Failed to open output file {}", output_file.display()))?;
            file_handle.write_all(buf.as_mut())
                .chain_err(|| format!("Failed to save to output file {}", output_file.display()))?;

        }

        if !matches.is_present("modtimes") && file.timestamp != 0 {
            let metadata = fs::metadata(output_file.clone()).unwrap();
            filetime::set_file_times(output_file.clone(),
                                     FileTime::from_last_access_time(&metadata),
                                     FileTime::from_seconds_since_1970(file.timestamp as u64, 0))
                      .chain_err(|| format!("Failed to set timestamp on file {}", output_file.display()))?;
        }

    }
    Ok(())
}
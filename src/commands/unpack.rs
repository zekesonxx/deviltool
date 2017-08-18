
use clap::ArgMatches;
use time::strptime;

use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use nom::IResult::*;
use nom::Needed::Size;
use filetime::{self, FileTime};

use super::super::types::*;
use super::super::parser;

pub fn execute(matches: &ArgMatches) {
    // make sure we have somewhere to put the files
    let mut output_dir = PathBuf::from(matches.value_of("FOLDER").unwrap());
    fs::create_dir_all(output_dir.clone());

    let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut reader = BufReader::new(f);
    let mut firstfolder = true;
    match parser::read_header(&mut reader).unwrap() {
        Ok((header, mut files)) => {
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
                        fs::create_dir_all(output_dir.clone());
                        continue;
                    }
                }
                let mut output_file = output_dir.join(file.filename.clone());
                output_file.set_extension(file.file_type.extension());

                {
                    reader.seek(SeekFrom::Start(file.offset as u64));
                    let mut buf = vec![0; file.size as usize];
                    reader.read_exact(&mut buf[..]);

                    if file.file_type == DDFiletype::GLSL && !matches.is_present("preserveglsl") {
                        match parser::glsl_file(buf.as_ref()) {
                            Incomplete(_) | Error(_) => {
                                println!("Malformed GLSL file! Saving as normal file");
                            },
                            Done(_, (name, vertex, fragment)) => {
                                if name != file.filename {
                                    println!("Warning: GLSL name is {} but saving as {}",
                                             name, file.filename);
                                }
                                output_file.set_extension("vert");
                                println!("Writing {}", output_file.display());
                                let mut file_handle = File::create(output_file.clone()).unwrap();
                                file_handle.write_all(vertex.as_bytes());

                                output_file.set_extension("frag");
                                println!("Writing {}", output_file.display());
                                let mut file_handle = File::create(output_file.clone()).unwrap();
                                file_handle.write_all(fragment.as_bytes());
                                continue;
                            }
                        }
                    }

                    println!("Writing {}", output_file.display());
                    let mut file_handle = File::create(output_file.clone()).unwrap();
                    file_handle.write_all(buf.as_mut());

                }

                if !matches.is_present("modtimes") && file.timestamp != 0 {
                    let metadata = fs::metadata(output_file.clone()).unwrap();
                    filetime::set_file_times(output_file.clone(),
                                             FileTime::from_last_access_time(&metadata),
                                             FileTime::from_seconds_since_1970(file.timestamp as u64, 0));
                }

            }
        },
        Err(err) => {
            println!("Error! {:?}", err);
            return;
        }
    }
}
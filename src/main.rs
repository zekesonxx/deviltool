#[macro_use] extern crate nom;
#[macro_use] extern crate clap;
extern crate time;
extern crate filetime;
extern crate image;
extern crate byteorder;

pub mod parser;
pub mod tex2;
use parser::*;

use nom::IResult::*;
use nom::Needed::Size;

use image::{GenericImage,ImageBuffer};

use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let file_exists = |path| {
        if std::fs::metadata(path).is_ok() {
            Ok(())
        } else {
            Err(String::from("File doesn't exist"))
        }
    };
    let file_still_exists = |path| {
        if std::fs::metadata(path).is_ok() {
            Ok(())
        } else {
            Err(String::from("File doesn't exist"))
        }
    };
    let file_still_really_exists = |path| {
        if std::fs::metadata(path).is_ok() {
            Ok(())
        } else {
            Err(String::from("File doesn't exist"))
        }
    };

    let matches: clap::ArgMatches = clap_app!(myapp =>
        (@setting ArgRequiredElseHelp)
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Manipulate data files from Devil Daggers")
        (@subcommand info =>
            (about: "Prints file list from an archive")
            (@setting ArgRequiredElseHelp)
            (@arg FILE: +required {file_exists} "File to print information about")
            (@arg types: -t --types "Print file types")
            (@arg extensions: -e --ext "Include guessed file extensions")
            (@arg verbose: -v --verbose "Print all information")
        )
        (@subcommand extract =>
            (about: "Extract files from an archive to a folder")
            (@setting ArgRequiredElseHelp)
            (@arg FILE: +required {file_still_exists} "File to extract")
            (@arg FOLDER: +required "Folder to extract to")
            (@arg modtimes: -m --nomodtimes "Don't export file modification times")
            (@arg nofolders: -f --nofolders "Don't automatically create subfolders for output")
            (@arg preserveglsl: -g --preserveglsl "Don't split GLSL shaders into their respective files")
        )
        (@subcommand imgconv =>
            (about: "Convert images to/from dd_tex2")
            (@setting ArgRequiredElseHelp)
            (@arg FILE: +required {file_still_really_exists} "File to convert")
            (@arg OUTFILE: "File to output to")
            //(@arg reverse: -r --reverse "Convert to tex2. Yes this is awkward.")
        )
    ).get_matches();

    match matches.subcommand() {
        ("info", Some(matches)) => {
            let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
            let mut reader = BufReader::new(f);
            match parser::read_header(&mut reader) {
                Ok(data) => {
                    let header: DDMainHeader = data.0;
                    let files: Vec<DDSubFileHeader> = data.1;
                    if matches.is_present("verbose") {
                        println!("## HEADER");
                        println!("magic bytes: {:?}", header.magic_number);
                        println!("header length: {}", header.header_length);
                        println!("## FILES");
                    } else {
                        println!("{} file{}", files.len(), if files.len() == 1 {""} else {"s"});
                    }

                    for file in files {
                        if matches.is_present("verbose") {
                            println!("offset {} B\tdatetime {}\t{} B\t{}\t{:?}",
                                     file.offset,
                                     time::strptime(format!("{}", file.timestamp).as_mut_str(), "%s").unwrap().rfc3339(),
                                     file.size,
                                     file.filename,
                                     file.file_type
                            );
                        } else if matches.is_present("types") {
                            println!("{}.{}: {} MB, {}",
                                     file.filename,
                                     if matches.is_present("extensions") {file.file_type.extension()} else {"".to_string()},
                                     (file.size as f32)/1024f32/1024f32,
                                     file.file_type
                            );
                        } else {
                            println!("{}: {} MB",
                                     file.filename,
                                     (file.size as f32)/1024f32/1024f32,
                            );
                        }
                    }
                },
                Err(err) => {
                    println!("Error! {:?}", err);
                    return;
                }
            }
        },
        ("extract", Some(matches)) => {
            // make sure we have somewhere to put the files
            let mut output_dir = PathBuf::from(matches.value_of("FOLDER").unwrap());
            std::fs::create_dir_all(output_dir.clone());

            let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
            let mut reader = BufReader::new(f);
            let mut firstfolder = true;
            match parser::read_header(&mut reader) {
                Ok(data) => {
                    let header: DDMainHeader = data.0;
                    let mut files: Vec<DDSubFileHeader> = data.1;
                    if !matches.is_present("nofolders") { files.reverse(); }
                    for file in files {
                        if file.file_type == DDFiletype::FolderMarker {
                            if matches.is_present("nofolders") {
                                println!("Ignoring folder marker {}", file.filename);
                            } else {
                                if !firstfolder {
                                    output_dir.pop();
                                } else {
                                    firstfolder = false;
                                }
                                output_dir.push(file.filename.clone());
                                std::fs::create_dir_all(output_dir.clone());
                            }
                            continue;
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
                            let metadata = std::fs::metadata(output_file.clone()).unwrap();
                            filetime::set_file_times(output_file.clone(),
                                                     filetime::FileTime::from_last_access_time(&metadata),
                                                     filetime::FileTime::from_seconds_since_1970(file.timestamp as u64, 0));
                        }

                    }
                },
                Err(err) => {
                    println!("Error! {:?}", err);
                    return;
                }
            }
        },
        ("imgconv", Some(matches)) => {
                let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
                let mut reader = BufReader::new(f);
                let mut buf: Vec<u8> = Vec::with_capacity(5000);
                reader.read_to_end(&mut buf);
                match tex2::tex2_image(buf.as_ref()) {
                    Done(unused, tex2image) => {
                        println!("parsed!");

                        let mut img = ImageBuffer::new(tex2image.width, tex2image.height);
                        img.copy_from(&tex2image, 0, 0);

                        let mut output_file = PathBuf::from(matches.value_of("FILE").unwrap());
                        output_file.set_extension("png");

                        let ref mut fout = File::create(output_file).unwrap();

                        // We must indicate the image’s color type and what format to save as
                        let _ = image::ImageRgba8(img).save(fout, image::PNG);
                    },
                    Error(err) => {
                        println!("error: {:?}", err);
                        println!("{}", err.description());
                    },
                    Incomplete(needed) => {
                        println!("need {:?} more bytes", needed);
                    }
                }
        }
        (_, _) => {}
    }
}

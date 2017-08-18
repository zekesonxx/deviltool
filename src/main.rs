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

use byteorder::{LittleEndian, WriteBytesExt};

use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::fs::File;
use std::path::{Path, PathBuf};

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
            (@arg foldermarkers: -k --foldermarkers "Export .foldermarker files instead of folders")
            (@arg preserveglsl: -g --preserveglsl "Don't split GLSL shaders into their respective files")
        )
        (@subcommand imgconv =>
            (about: "Convert images to/from dd_tex2")
            (@setting ArgRequiredElseHelp)
            (@arg FILE: +required {file_still_really_exists} "File to convert")
            (@arg OUTFILE: "File to output to")
            //(@arg reverse: -r --reverse "Convert to tex2. Yes this is awkward.")
        )
        (@subcommand pack =>
            (about: "Pack a directory of files into a dd-format archive")
            (@setting ArgRequiredElseHelp)
            (@arg ARCHIVE: +required "Archive to output to")
            (@arg DIR: +required "Directory to get files from")
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
                                std::fs::create_dir_all(output_dir.clone());
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
        },
        ("pack", Some(matches)) => {
            let folder = PathBuf::from(matches.value_of("DIR").unwrap());
            if !folder.is_dir() {
                //TODO this is a horrible error message
                println!("You need to pass us a folder!");
                return;
            }

            // Array of files that will be in the header
            let mut files: Vec<(PathBuf, DDSubFileHeader)> = vec![];

            // Length of all of the subheaders
            // Used later to calculate offsets
            // Starts at 2 due to the 2 null bytes at the end of the header
            let mut total_subheader_length: u32 = 2;

            // Biggest file we'll need to load into memory
            //TODO replace this with a smaller buffer in a loop
            let mut biggest_file_size: usize = 0;

            let mut iter = folder.read_dir().unwrap();
            println!("## Building file list");
            for file in iter {
                let file = file.unwrap();
                let filesize = file.metadata().unwrap().len() as u32;
                let filepath = file.path();
                let filesize = file.metadata().unwrap().len() as u32;
                let mut filetype;

                // Determine saved filename
                let mut filename = file.path();
                filename.set_extension("");
                let filename = filename.file_name().unwrap().to_os_string().into_string().unwrap();

                // Determine file type

                if let Some(ext) = filepath.extension() {
                    match DDFiletype::from_extension(ext.to_str().unwrap()) {
                        Some(newtype) => {
                            filetype = newtype;
                        },
                        None => {
                            println!("{}: Unrecognized file type {:?}", filepath.display(), ext);
                            println!("TODO: implement .dd_0xXX detection");
                            return;
                        }
                    }
                } else {
                    println!("{} has no extension, so we can't determine its file type.", filepath.display());
                    println!("If you need to pass a custom type, use .dd_0xXX, where XX is a number between 00 and FF.");
                    return;
                }

                // Determine Timestamp
                let mtime = filetime::FileTime::from_last_modification_time(&file.metadata().unwrap()).seconds_relative_to_1970() as u32;

                //TODO remove this because it's dumb
                if filesize as usize > biggest_file_size {
                    biggest_file_size = filesize as usize;
                }

                // Give an update to the user
                println!("{}: {}, {}B",
                         filepath.display(),
                         filetype,
                         filesize
                );

                // Increment the subheader length
                // filetype(u16) + filename with null term + offset(u32) + size(u32) + timestamp(u32)
                total_subheader_length += 2u32 + (filename.len() as u32 + 1) + 4 + 4 + 4;

                // Finally, save the data away
                files.push((filepath, DDSubFileHeader {
                    filename: filename,
                    file_type: filetype,
                    timestamp: mtime,
                    size: filesize,
                    offset: 0
                }));

            }
            println!("## Built list of {} file{}", files.len(), if files.len() == 1 {""} else {"s"});
            println!("Total subheader length: {}B", total_subheader_length);
            let files_start_at: u32 = total_subheader_length + 12;
            println!("First file offset at: {}", files_start_at);


            println!("Beginning file output");
            let mut output_archive = BufWriter::new(File::create(matches.value_of("ARCHIVE").unwrap()).unwrap());

            //TODO switch this to a struct
            output_archive.write(b":hx:rg:\x01");
            output_archive.write_u32::<LittleEndian>(total_subheader_length);
            println!("Wrote main header");

            // Export the whole subheader section
            let mut cur_offset: u32 = files_start_at;
            for &mut (_, ref mut subheader) in files.iter_mut() {
                subheader.offset = cur_offset;
                subheader.write(&mut output_archive);
                cur_offset += subheader.size;
            }
            // Double null byte to signify header end
            output_archive.write(&[0, 0]);
            println!("Wrote subheader");

            // Let's start reading files!
            // Preallocate ~60MB because avoid reallocs
            let mut buf = Vec::with_capacity(biggest_file_size);
            for (filepath, subheader) in files {
                let mut reader = File::open(filepath.clone()).unwrap();
                let result = reader.read_to_end(&mut buf);
                if let Ok(a) = result {
                    if a as u32 != subheader.size {
                        println!("Something changed the file {}!", filepath.display());
                        println!("The size isn't the same anymore!");
                        return;
                    }
                } else {
                    println!("Failed to read file {}", filepath.display());
                    return;
                }
                output_archive.seek(SeekFrom::Start(subheader.offset as u64));
                output_archive.write_all(&mut buf.as_ref());
                buf.clear();
                println!("Wrote {}", subheader.filename);
            }

            println!("Built archive {}", matches.value_of("ARCHIVE").unwrap());


        }
        (_, _) => {}
    }
}

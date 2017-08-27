
use clap::ArgMatches;
use filetime::FileTime;

use std::io::prelude::*;
use std::io::{BufWriter, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use byteorder::{LittleEndian, WriteBytesExt};

use super::super::types::{DDSubFileHeader, DDFiletype};

pub fn execute(matches: &ArgMatches) {
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

    let iter = folder.read_dir().unwrap();
    println!("## Building file list");
    for file in iter {
        let file = file.unwrap();
        let filesize = file.metadata().unwrap().len() as u32;
        let filepath = file.path();
        let filetype;

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
        let mtime = if matches.is_present("zerotime") {
            0u32
        } else {
            FileTime::from_last_modification_time(&file.metadata().unwrap()).seconds_relative_to_1970() as u32
        };

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

    // Sort file list alphabetically
    // This makes packing deterministic.
    // (no relying on the semi-random order the FS gives them to us)
    files.sort_by(|a, b| a.1.filename.cmp(&b.1.filename));
    println!("Sorted file list.");

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
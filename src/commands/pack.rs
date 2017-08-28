
use clap::ArgMatches;
use filetime::FileTime;

use std::io::prelude::*;
use std::io::{BufWriter, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use byteorder::{LittleEndian, WriteBytesExt};

use super::super::types::{DDSubFileHeader, DDFiletype};
use super::super::errors::*;

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let folder = PathBuf::from(matches.value_of("DIR").unwrap());
    if !folder.is_dir() {
        //TODO this is a horrible error message
        panic!("This is an error message that shouldn't happen but does because clap");
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

    let iter = folder.read_dir().chain_err(|| "Failed to read file list from directory")?;
    println!("## Building file list");
    for file in iter {
        let file = file.unwrap();
        let metadata = file.metadata().chain_err(|| "Failed to read file metadata")?;
        let filesize = metadata.len() as u32;
        let filepath = file.path();
        let filetype;

        // Determine saved filename
        let mut filename = file.path();
        filename.set_extension("");
        let filename = String::from(filename.file_name().unwrap().to_string_lossy());

        // Determine file type

        if let Some(ext) = filepath.extension() {
            match DDFiletype::from_extension(&ext.to_string_lossy()) {
                Some(newtype) => {
                    filetype = newtype;
                },
                None => {
                    println!("{}: Unrecognized file type {:?}", filepath.display(), ext);
                    println!("TODO: implement .dd_0xXX detection");
                    panic!(); //TODO this
                }
            }
        } else {
            println!("{} has no extension, so we can't determine its file type.", filepath.display());
            println!("If you need to pass a custom type, use .dd_0xXX, where XX is a number between 00 and FF.");
            panic!(); //TODO this
        }

        // Determine Timestamp
        let mtime = if matches.is_present("zerotime") {
            0u32
        } else {
            FileTime::from_last_modification_time(&metadata).seconds_relative_to_1970() as u32
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
    let mut output_archive = BufWriter::new(File::create(matches.value_of("ARCHIVE").unwrap())
        .chain_err(|| "Failed to open output archive")?);

    //TODO switch this to a struct
    output_archive.write(b":hx:rg:\x01")
        .chain_err(|| "Failed to write magic bytes to output file")?;
    output_archive.write_u32::<LittleEndian>(total_subheader_length)
        .chain_err(|| "Failed to write header start to output file")?;
    println!("Wrote main header");

    // Export the whole subheader section
    let mut cur_offset: u32 = files_start_at;
    for &mut (_, ref mut subheader) in files.iter_mut() {
        subheader.offset = cur_offset;
        subheader.write(&mut output_archive)
            .chain_err(|| format!("Failed to write header for file {} to output archive", subheader.filename))?;
        cur_offset += subheader.size;
    }
    // Double null byte to signify header end
    output_archive.write(&[0, 0]).chain_err(|| "Failed to write header end to output file")?;
    println!("Wrote subheader");

    // Let's start reading files!
    // Preallocate the size of the biggest file because avoid reallocs
    let mut buf = Vec::with_capacity(biggest_file_size);
    for (filepath, subheader) in files {
        let mut reader = File::open(filepath.clone())
            .chain_err(|| format!("Failed to open file {}", filepath.display()))?;
        let bytes_read = reader.read_to_end(&mut buf)
            .chain_err(|| format!("Failed to read file {}", filepath.display()))?;

        if bytes_read as u32 != subheader.size {
            return Err(format!("Something changed the file {} very quickly. {}", filepath.display(),
                               "We detect and deny this to prevent a race condition.").into());
        }

        output_archive.seek(SeekFrom::Start(subheader.offset as u64))
            .chain_err(|| "Failed to seek within output file")?;
        output_archive.write_all(&mut buf.as_ref())
            .chain_err(|| "Failed to write to output file")?;
        buf.clear();
        println!("Wrote {}", subheader.filename);
    }

    println!("Built archive {}", matches.value_of("ARCHIVE").unwrap());
    Ok(())
}
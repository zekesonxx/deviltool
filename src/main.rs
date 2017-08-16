#[macro_use] extern crate nom;
#[macro_use] extern crate clap;
extern crate time;

pub mod parser;
use parser::*;

use nom::IResult::*;
use nom::Needed::Size;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

fn main() {
    let file_exists = |path| {
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
            (@arg verbose: -v --verbose "Print all information")
        )
    ).get_matches();

    match matches.subcommand() {
        ("info", Some(matches)) => {
            let f = File::open(matches.value_of("FILE").unwrap()).unwrap();
            let mut reader = BufReader::new(f);
            let mut header: Vec<u8> = (0..12).collect::<Vec<_>>();
            reader.read_exact(&mut header[..12]);
            match header_section_bound(header.as_ref()) {
                Incomplete(Size(size)) => {
                    reader.take(size as u64).read_to_end(&mut header);
                    match header_section_bound(header.as_ref()) {
                        Done(_, data) => {
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
                                    println!("{}: {} MB, {}",
                                             file.filename,
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
                        Incomplete(_) => {
                            println!("Read Error");
                            return;
                        },
                        Error(err) => {
                            println!("Parse error {:?}", err);
                        }
                    }
                },
                _ => {
                    println!("Horribly malformed file, or no contents");
                    return;
                }
            }

        },
        (_, _) => {}
    }
}

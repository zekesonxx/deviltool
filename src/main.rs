#[macro_use] extern crate nom;
#[macro_use] extern crate clap;
extern crate time;
extern crate filetime;
extern crate image;
extern crate byteorder;

pub mod parser;
pub mod tex2;
mod commands;
use parser::*;

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
        (@subcommand unpack =>
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
            (about: "Convert images from dd_tex2 to png")
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
        ("info", Some(matches)) => commands::info::execute(matches),
        ("unpack", Some(matches)) => commands::unpack::execute(matches),
        ("imgconv", Some(matches)) => commands::imgconv::execute(matches),
        ("pack", Some(matches)) => commands::pack::execute(matches),
        (_, _) => {}
    }
}

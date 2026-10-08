use ecm3428::{decode, encode};

use std::env;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;

const APP_NAME: &str = "APP NAME";
const COMMANDS: [&str; 3] = ["enc", "dec", "help"];

// contains the code that will allow the CLI to run
fn main() {
    // CLI args stored as a Vector to allow array-like operations
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    // Program exits if no command was given
    if argc == 1 {
        command_error("No command given.");
    };

    // Changes plurality depending on the number of arguments
    let arg_word: &str = if argc == 2 { "was" } else { "were" };

    // Initialises the command the user entered and checks if it is within the array of commands
    let command: &str = args[1].as_str();
    if !COMMANDS.contains(&command) {
        command_error(&format!("Unknown command {}.", command));
    };

    // prints help instructions on screen as long as the correct num of args given
    if command == "help" {
        if argc != 2 {
            command_error(&format!(
                "Expected 1 argument but {} {} given.",
                argc - 1,
                arg_word
            ));
        } else {
            println!("Usage: {} [COMMAND] [ARGUMENTS]", APP_NAME);
            println!(concat!(
                "Commands:\n",
                "   help                        Help about commands.\n",
                "   enc [SRC PATH] [OUT PATH]   Encodes the file given in SRC PATH and outputs the result to OUT PATH.\n",
                "   dec [SRC PATH] [OUT PATH]   Decodes the file given in SRC PATH and outputs the result to OUT PATH.\n"
            ));
            std::process::exit(0);
        }
    }

    // TODO
    if command == "enc" {
        if argc != 4 {
            command_error(&format!(
                "Expected 3 arguments but {} {} given.",
                argc - 1,
                arg_word
            ));
        };

        // checks if filepaths are valid before encoding file
        let src: &Path = Path::new(&args[2]);
        let des: &Path = Path::new(&args[3]);

        if des.is_file() {
            // asks user if destination file can be overwritten
            check_des_overwritable();
        };

        // runs encode function and catches any errors
        match encode(src, des) {
            Ok(()) => {}
            Err(error) => {
                // matches all possible IO errors
                match_error(error);
            }
        }
    }

    // TODO
    if command == "dec" {
        if argc != 4 {
            command_error(&format!(
                "Expected 3 arguments but {} {} given.",
                argc - 1,
                arg_word
            ));
        };

        // checks if filepaths are valid before decoding file
        let src: &Path = Path::new(&args[2]);
        let des: &Path = Path::new(&args[3]);

        if des.is_file() {
            // asks user if destination file can be overwritten
            check_des_overwritable();
        };

        // runs decode function and catches any errors
        match decode(src, des) {
            Ok(()) => {}
            Err(error) => {
                // matches all possible IO errors
                match_error(error);
            }
        }
    }
}

// Function for formatting CLI errors
fn command_error(error_message: &str) -> ! {
    eprintln!("Error: {} (Try {} help)", error_message, APP_NAME);
    std::process::exit(1);
}

fn check_des_overwritable() {
    println!("The destination path is a file. Would you like to overwrite it? (y/n): ");
    let mut answer: String = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Could not read input");
    let mut ans_low: String = answer.trim().to_lowercase();

    while ans_low != "y" && ans_low != "yes" {
        if ans_low == "n" || ans_low == "no" {
            std::process::exit(0);
        }

        println!("Please either enter 'y' or 'n':");
        answer.clear();

        io::stdin()
            .read_line(&mut answer)
            .expect("Could not read input");
        ans_low = answer.trim().to_lowercase();
    }
}

fn match_error(error: io::Error) {
    match error.kind() {
        ErrorKind::NotFound => {
            command_error("File could not be found.");
        }
        ErrorKind::IsADirectory => {
            command_error("A file was expected, but a directory was found.");
        }
        ErrorKind::ReadOnlyFilesystem => {
            command_error("The destination filesystem is read-only.");
        }
        ErrorKind::WriteZero => {
            command_error("The program was unable to write data to the destination.");
        }
        ErrorKind::StorageFull => {
            command_error("The destination storage is full.");
        }
        ErrorKind::QuotaExceeded => {
            command_error("The filesystem storage quota has been exceeded.");
        }
        ErrorKind::FileTooLarge => {
            command_error("The file is too large.");
        }
        ErrorKind::ResourceBusy => {
            command_error("The file or resource is currently busy.");
        }
        ErrorKind::InvalidFilename => {
            command_error("The filename is invalid.");
        }
        ErrorKind::Interrupted => {
            command_error("The file operation was interrupted.");
        }
        ErrorKind::UnexpectedEof => {
            command_error("The file ended unexpectedly while being read.");
        }
        ErrorKind::OutOfMemory => {
            command_error("There was not enough memory to complete the operation.");
        }
        ErrorKind::Unsupported => {
            command_error("The version of the file is more recent than the decoder version that is being used.");
        }
        ErrorKind::PermissionDenied => {
            command_error("The file could not be accessed due to its permissions.");
        }
        _ => {
            command_error(&format!("An I/O error occurred: {}", error));
        }
    };
}

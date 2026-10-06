use ecm3428::{decode, encode};

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::io;

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
        } else {
            // checks if filepaths are valid before encoding file
            let src: &Path = Path::new(&args[2]);
            let des: &Path = Path::new(&args[3]);

            if !src.is_file() {
                command_error(&format!("Encode src path does not lead to a file."));
            }

            if fs::read(src).is_err() {
                command_error(&format!("Encode src path cannot be read."));
            };

            if !des.exists() {
                command_error(&format!("Encode des path does not exist."));
            };

            // checks to see if directory can be written in
            if des.is_dir() {
                let test_file = des.join("write");

                if fs::write(&test_file, "").is_err() {
                    command_error("Encode des path cannot be written to");
                }

                // temporary file removed after check
                fs::remove_file(&test_file).expect("Encode temp writing file could not be removed");
            } else if des.is_file() {
                check_des_overwritable();
            };

            // checks for errors when encoding
            if let Err(error) = encode(src, des) {
                command_error(&format!("Failed to encode file: {}", error));
            };
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
        } else {
            // checks if filepaths are valid before decoding file
            let src: &Path = Path::new(&args[2]);
            let des: &Path = Path::new(&args[3]);

            if !src.is_file() {
                command_error(&format!("Decode src path does not lead to a file."));
            }

            if fs::read(src).is_err() {
                command_error(&format!("Decode src path cannot be read."));
            };

            if !des.exists() {
                command_error(&format!("Decode des path does not exist."));
            };

            // checks to see if directory can be written in
            if des.is_dir() {
                let test_file = des.join("write");

                if fs::write(&test_file, "").is_err() {
                    command_error("Decode des path cannot be written to");
                }

                // temporary file removed after check
                fs::remove_file(&test_file).expect("Decode temp writing file could not be removed");
            } else if des.is_file() {
                // asks user if destination file can be overwritten
                check_des_overwritable();
            };

            // checks for errors when decoding
            if let Err(error) = decode(src, des) {
                command_error(&format!("Failed to decode file: {}", error));
            };
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
    let ans_low: &str = &answer.trim().to_lowercase();

    if ans_low == "n" || ans_low == "no" {
        std::process::exit(0);
    } else if ans_low != "y" && ans_low != "yes" {
        command_error("Answer given was not recognised");
    };
}

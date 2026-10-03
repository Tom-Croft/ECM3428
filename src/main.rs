use ecm3428::{decode, encode};

use std::env;

const APP_NAME: &str = "APP NAME";
const COMMANDS: [&str; 3] = ["enc", "dec", "help"];

// contains the code that will allow the CLI to run
fn main() {
    // CLI args stored as a Vector to allow array-like operations
    let args: Vec<String> = env::args().collect();
    let argc = args.len();
    let command: &str;
    let arg_word: &str;

    // get rid of this l8r
    println!("{:?}", args);
    println!("{}", argc);

    // Program exits if no command was given
    if argc == 1 {
        command_error("No command given.");
    }

    // Changes plurality depending on the number of arguments
    arg_word = if argc == 2 { "was" } else { "were" };

    // Initialises the command the user entered and checks if it is within the array of commands
    if COMMANDS.contains(&args[1].as_str()) {
        command = args[1].as_str();
    } else {
        command = ""; // must initialise 'command' before it is used in the next if-statement or else error
        command_error(format!("Unknown command {}.", args[1].as_str()).as_str());
    }

    // prints help instructions on screen as long as the correct num of args given
    if command == "help" {
        if argc != 2 {
            command_error(
                format!("Expected 1 argument but {} {} given.", argc - 1, arg_word).as_str(),
            );
        } else {
            println!("{}", format!("Usage: {} [COMMAND] [ARGUMENTS]", APP_NAME));
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
            return command_error(
                format!("Expected 3 arguments but {} {} given.", argc - 1, arg_word).as_str(),
            );
        } else {
            println!("TODO encode");
            std::process::exit(0);
        }
    }

    // TODO
    if command == "dec" {
        if argc != 4 {
            return command_error(
                format!("Expected 3 arguments but {} {} given.", argc - 1, arg_word).as_str(),
            );
        } else {
            println!("TODO decode");
            std::process::exit(0);
        }
    }

    unreachable!("Should be unreachable")
}

// Function for formatting CLI errors
fn command_error(error_message: &str) {
    eprintln!(
        "{}",
        format!("Error: {} (Try {} help)", error_message, APP_NAME)
    );
    std::process::exit(1);
}

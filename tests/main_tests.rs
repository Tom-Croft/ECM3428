use ecm3428::*;

use std::fs;
use std::io::ErrorKind;
use std::process::Command;

// creates a temporary directory for integration tests
fn temporary_directory() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("ecm3428_test");
    fs::create_dir_all(&dir).unwrap();
    return dir;
}

// links the CL args
fn run_command(args: &[&str]) -> std::process::Output {
    return Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(args)
        .output()
        .expect("Error");
}

// helper that converts the standard err stream into a readable string
fn error_msg(output: &std::process::Output) -> &str {
    return std::str::from_utf8(&output.stderr).unwrap();
}

// tests if the program throws an error if no args are given in the CLI
#[test]
fn test_no_arg_given() {
    let output = run_command(&[]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("No command given."));
}

// checks for an error if an unknown command is used as an arg
#[test]
fn test_command_unknown() {
    let output = run_command(&["encoding"]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("Unknown command encoding."));
}

// makes sure that writing the commands in the wrong case returns an error
#[test]
fn test_help_case_sensitive() {
    let output = run_command(&["HELP"]);
    assert!(!output.status.success());
}

// tests if the program returns an error for the wrong num of args for 'help'
#[test]
fn test_help_wrong_args() {
    let output = run_command(&["help", "42069"]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("Expected 1 argument but 2 were given."));
}

// checks if using the 'help' command does not result in any errors
#[test]
fn test_help_right_args() {
    let output = run_command(&["help"]);
    assert!(output.status.success());
}

#[test]
fn test_enc_wrong_args() {
    let output = run_command(&["enc"]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("Expected 3 arguments but 1 was given."));
}

#[test]
fn test_dec_wrong_args() {
    let output = run_command(&["dec"]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("Expected 3 arguments but 1 was given."));
}

#[test]
fn test_freq_wrong_args() {
    let output = run_command(&["freq"]);
    assert!(!output.status.success());
    assert!(error_msg(&output).contains("Expected 2 arguments but 1 was given."));
}

// frequency analysis only works on an encoded file, so without it we have to check for an error
#[test]
fn test_freq_path_empty() {
    let output = run_command(&["freq", ""]);
    assert!(!output.status.success());
}

// A module that contains a copy of the 'main.rs' private functions that can be used for testing error-handling
mod main {
    include!("../src/main.rs");

    // allows us to use 'match_error' from 'main.rs' with any error we want to test
    pub fn run_error(kind: std::io::ErrorKind) {
        match_error(std::io::Error::from(kind));
    }
}

// helper method that checks if error-handling works correctly
fn check_error(error: &str, msg: &str) {
    // we change the environment for the 'test_match_error' test so that it can access the error
    // the '--nocapture' arg means the program won't terminate before we can test everything
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "test_match_error", "--nocapture"])
        .env("TEST_ERROR", error)
        .output()
        .unwrap();

    // Since we are testing errors, the program should fail and print out the appropriate message
    assert!(!output.status.success());
    assert!(error_msg(&output).contains(msg));
}

// a test that can differentiate between different types of errors to see if they are handled correctly
#[test]
fn test_match_error() {
    // error type is fetched from the environment, which we defined in 'check_error'
    let error = std::env::var("TEST_ERROR");

    if error.is_err() {
        return;
    }

    // we match the name to the actual 'ErrorKind' variable
    let kind = match error.unwrap().as_str() {
        "NotFound" => ErrorKind::NotFound,
        "IsADirectory" => ErrorKind::IsADirectory,
        "ReadOnlyFilesystem" => ErrorKind::ReadOnlyFilesystem,
        "WriteZero" => ErrorKind::WriteZero,
        "StorageFull" => ErrorKind::StorageFull,
        "QuotaExceeded" => ErrorKind::QuotaExceeded,
        "FileTooLarge" => ErrorKind::FileTooLarge,
        "ResourceBusy" => ErrorKind::ResourceBusy,
        "InvalidFilename" => ErrorKind::InvalidFilename,
        "Interrupted" => ErrorKind::Interrupted,
        "UnexpectedEof" => ErrorKind::UnexpectedEof,
        "OutOfMemory" => ErrorKind::OutOfMemory,
        "Unsupported" => ErrorKind::Unsupported,
        "PermissionDenied" => ErrorKind::PermissionDenied,
        "Other" => ErrorKind::Other,
        _ => ErrorKind::Other,
    };

    // we check what the error returns by using the module to run the 'main.rs' code
    main::run_error(kind);
}

// All of the 'error' tests see if the correct error-handling happens for each error we try to catch
#[test]
fn test_error_not_found() {
    check_error("NotFound", "File could not be found.");
}

#[test]
fn test_error_is_directory() {
    check_error(
        "IsADirectory",
        "A file was expected, but a directory was found.",
    );
}

#[test]
fn test_error_read_only() {
    check_error(
        "ReadOnlyFilesystem",
        "The destination filesystem is read-only.",
    );
}

#[test]
fn test_error_write_zero() {
    check_error(
        "WriteZero",
        "The program was unable to write data to the destination.",
    );
}

#[test]
fn test_error_storage_full() {
    check_error("StorageFull", "The destination storage is full.");
}

#[test]
fn test_error_quota_exceeded() {
    check_error(
        "QuotaExceeded",
        "The filesystem storage quota has been exceeded.",
    );
}

#[test]
fn test_error_file_too_large() {
    check_error("FileTooLarge", "The file is too large.");
}

#[test]
fn test_error_resource_busy() {
    check_error("ResourceBusy", "The file or resource is currently busy.");
}

#[test]
fn test_error_invalid_filename() {
    check_error("InvalidFilename", "The filename is invalid.");
}

#[test]
fn test_error_interrupted() {
    check_error("Interrupted", "The file operation was interrupted.");
}

#[test]
fn test_error_unexpected_eof() {
    check_error(
        "UnexpectedEof",
        "The file ended unexpectedly while being read.",
    );
}

#[test]
fn test_error_out_of_memory() {
    check_error(
        "OutOfMemory",
        "There was not enough memory to complete the operation.",
    );
}

#[test]
fn test_error_unsupported() {
    check_error(
        "Unsupported",
        "The version of the file is more recent than the decoder version that is being used.",
    );
}

#[test]
fn test_error_permission_denied() {
    check_error(
        "PermissionDenied",
        "The file could not be accessed due to its permissions.",
    );
}

#[test]
fn test_error_other() {
    check_error("Other", "An I/O error occurred:");
}

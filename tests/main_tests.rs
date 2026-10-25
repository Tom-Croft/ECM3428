use ecm3428::*;

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn temporary_directory() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("ecm3428_test");
    fs::create_dir_all(&dir).unwrap();
    return dir;
}

fn run_command(args: &[&str]) -> std::process::Output {
    return Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(args)
        .output()
        .expect("Error");
}

#[test]
fn test_no_arg_given() {
    let output = run_command(&[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No command given."));
}

#[test]
fn test_command_empty() {
    let output = run_command(&[""]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown command"));
}

#[test]
fn test_command_unknown() {
    let output = run_command(&["encoding"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown command encoding."));
}

#[test]
fn test_help_case_sensitive() {
    let output = run_command(&["HELP"]);
    assert!(!output.status.success());
}

#[test]
fn test_help_wrong_args() {
    let output = run_command(&["help", "42069"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Expected 1 argument but 2 were given.")
    );
}

#[test]
fn test_help_right_args() {
    let output = run_command(&["help"]);
    assert!(output.status.success());
}

#[test]
fn test_enc_wrong_args() {
    let output = run_command(&["enc"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Expected 3 arguments but 1 was given.")
    );
}

#[test]
fn test_dec_wrong_args() {
    let output = run_command(&["dec"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Expected 3 arguments but 1 was given."));
}

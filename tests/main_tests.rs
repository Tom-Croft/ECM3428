use ecm3428::*;

use std::process::Command;

#[test]
fn test_no_arg_given() {
    let no_args = Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args([""])
        .output()
        .expect("No command given.");

    assert!(!no_args.status.success());
}

#[test]
fn test_help_wrong_args() {
    let help_wrong_args = Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(["help", "42069"])
        .output()
        .expect("Expected 1 argument but 2 were given.");

    assert!(!help_wrong_args.status.success());
}

#[test]
fn test_help_right_args() {
    let help_right_args = Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(["help"])
        .output()
        .expect("Help info should be printed");

    assert!(help_right_args.status.success());
}

#[test]
fn test_enc_wrong_args() {
    let enc_wrong_args = Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(["enc"])
        .output()
        .expect("Expected 3 arguments but 1 was given.");

    assert!(!enc_wrong_args.status.success());
}

#[test]
fn test_dec_wrong_args() {
    let dec_wrong_args = Command::new(env!("CARGO_BIN_EXE_ecm3428"))
        .args(["dec"])
        .output()
        .expect("Expected 3 arguments but 1 was given.");

    assert!(!dec_wrong_args.status.success());
}

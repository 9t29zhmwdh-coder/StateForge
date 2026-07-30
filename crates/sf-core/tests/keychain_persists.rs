// macOS only. On Linux keyring defaults to the D-Bus secret service, which no CI
// runner provides, and a failure there would say nothing about this code.
#![cfg(target_os = "macos")]

//! Checks that a stored secret survives the process that wrote it.
//!
//! `keyring` without a platform feature compiles and passes a store-then-read
//! test inside one process, because it falls back to a store held in process
//! memory. Nothing reaches the keychain and no error is raised, so a
//! same-process test is blind to exactly the defect this guards against.
//!
//! The check therefore re-runs this binary as a child process. Reading through
//! `/usr/bin/security` instead would also cross the process boundary, but the
//! keychain grants read access per application: a different binary asking for
//! an item it did not create raises an authorisation dialog, which blocks CI and
//! interrupts whoever is at the keyboard. The same binary carries the same
//! access and stays silent.

use std::process::Command;

const SERVICE: &str = "sf-keychain-test";
const CHILD_ENV: &str = "SF_KEYCHAIN_TEST_CHILD";

#[test]
fn a_stored_secret_survives_the_writing_process() {
    let secret = "correct horse battery staple";

    // Child role: read what the parent wrote and report through the exit code.
    if let Ok(account) = std::env::var(CHILD_ENV) {
        let entry = keyring::Entry::new(SERVICE, &account).expect("entry");
        match entry.get_password() {
            Ok(v) if v == secret => std::process::exit(0),
            Ok(_) => std::process::exit(2),
            Err(_) => std::process::exit(3),
        }
    }

    let account = format!("probe-{}", std::process::id());
    let entry = keyring::Entry::new(SERVICE, &account).expect("entry");
    entry.set_password(secret).expect("set_password");

    let status = Command::new(std::env::current_exe().expect("own path"))
        .args(["--exact", "a_stored_secret_survives_the_writing_process", "--nocapture"])
        .env(CHILD_ENV, &account)
        .status()
        .expect("run child");

    let _ = entry.delete_credential();

    match status.code() {
        Some(0) => {}
        Some(2) => panic!("the keychain returned a different value"),
        Some(3) => panic!(
            "nothing persisted: keyring fell back to an in-memory store, \
             which means no platform feature is enabled"
        ),
        other => panic!("child exited unexpectedly: {other:?}"),
    }
}

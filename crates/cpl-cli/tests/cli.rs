use std::process::Command;

#[test]
fn binary_reports_its_version() {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("--version")
        .output()
        .expect("cpl runs");
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("cpl {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn binary_rejects_unknown_commands() {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("frobnicate")
        .output()
        .expect("cpl runs");
    assert_eq!(out.status.code(), Some(2));
}

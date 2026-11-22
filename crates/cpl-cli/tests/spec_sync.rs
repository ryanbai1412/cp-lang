//! `cpl --help` and the command table in `spec/tools/cli.md` name the same
//! commands, except a known list that may only shrink. Q tools-1 in
//! plan/QUESTIONS.md decides how each one is resolved.

use std::collections::BTreeSet;

/// In the spec, not in the binary.
const SPEC_ONLY: &[&str] = &[
    "debug", "fetch", "interact", "locate", "migrate", "selftest", "trace",
];
/// In the binary, not in the spec.
const CLI_ONLY: &[&str] = &["check", "dap", "fix", "judge", "replay", "report", "self"];

fn spec_commands() -> BTreeSet<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/tools/cli.md");
    let spec = std::fs::read_to_string(path).unwrap();
    spec.lines()
        .filter_map(|line| line.strip_prefix("| `cpl "))
        .map(|rest| {
            rest.split(|c: char| !c.is_ascii_lowercase())
                .next()
                .unwrap()
                .to_string()
        })
        .collect()
}

fn cli_commands() -> BTreeSet<String> {
    let help = String::from_utf8(cpl_cli::run(&["--help"]).stdout).unwrap();
    let list = help
        .split("commands:")
        .nth(1)
        .unwrap()
        .split("options:")
        .next()
        .unwrap();
    list.lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|rest| rest.starts_with(|c: char| c.is_ascii_lowercase()))
        .map(|rest| {
            rest.split(|c: char| !c.is_ascii_lowercase())
                .next()
                .unwrap()
                .to_string()
        })
        .collect()
}

fn names(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|name| name.to_string()).collect()
}

#[test]
fn the_spec_and_the_binary_name_the_same_commands_but_a_shrinking_list() {
    let spec = spec_commands();
    let cli = cli_commands();
    assert_eq!(cli.len(), 23, "{cli:?}");
    let spec_only: BTreeSet<String> = spec.difference(&cli).cloned().collect();
    let cli_only: BTreeSet<String> = cli.difference(&spec).cloned().collect();
    assert_eq!(
        spec_only,
        names(SPEC_ONLY),
        "a command the spec names and `cpl --help` does not; shrink SPEC_ONLY when one is resolved, never grow it"
    );
    assert_eq!(
        cli_only,
        names(CLI_ONLY),
        "a command `cpl --help` lists and the spec does not; shrink CLI_ONLY when one is resolved, never grow it"
    );
}

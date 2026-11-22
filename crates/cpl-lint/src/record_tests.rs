//! Every lint is a record under `spec/lints/<name>/`: `before.cp` fires,
//! `after.cp` and `near_miss.cp` do not.

use super::*;

fn codes(src: &str) -> Vec<&'static str> {
    lint(src, &Config::default())
        .iter()
        .map(|d| d.code.0)
        .collect()
}

#[test]
fn every_lint_record_fires_on_before_only() {
    for lint in LINTS {
        let code = lint.code.0;
        assert!(
            codes(lint.trigger).contains(&code),
            "{}: before.cp does not fire",
            lint.name
        );
        assert!(
            !codes(lint.fix).contains(&code),
            "{}: after.cp still fires",
            lint.name
        );
        assert!(
            !codes(lint.near_miss).contains(&code),
            "{}: near_miss.cp fires",
            lint.name
        );
    }
}

#[test]
fn every_lint_record_directory_belongs_to_a_lint() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/lints");
    for entry in std::fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            lint_named(&name).is_some(),
            "spec/lints/{name} names no lint"
        );
    }
}

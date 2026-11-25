#![cfg(feature = "cpp")]

use cpl_facade::{Job, Session};

/// spec/tools/explain.md: each index the file writes is listed with whether
/// it is checked; spec/tools/cli.md: one entry per choice, each with its site.
#[test]
fn each_index_the_file_writes_is_a_choice_at_its_place() {
    let source = "let xs = [1, 2, 3]\nlet i: Int = read()\nprint(xs[i])\n";
    let db = Session::default().with_file("p.cp", source);
    let program = (db.program("p.cp", &Job::program())).unwrap_or_else(|e| panic!("{e:?}"));
    let choices = program.choices(source);
    let indexes: Vec<(&str, &str)> = (choices.iter())
        .filter(|choice| choice.kind.starts_with("index_"))
        .map(|choice| (choice.site.as_str(), choice.kind))
        .collect();
    assert_eq!(indexes, [("3:7", "index_checked")], "{choices:?}");
    let report = program.explained("p.cp", source, None);
    assert!(report.contains("[check]"), "{report}");
}

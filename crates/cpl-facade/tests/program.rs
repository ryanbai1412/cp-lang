//! What a built program says of itself, with no lowered form in hand.

use cpl_facade::{Job, Program, Session};

fn built(source: &str) -> Program {
    let db = Session::default().with_file("p.cp", source);
    db.program("p.cp", &Job::program())
        .unwrap_or_else(|stopped| panic!("{stopped:?}"))
}

fn line(source: &str, at: u32) -> usize {
    source[..at as usize].matches('\n').count() + 1
}

#[test]
fn a_program_names_each_statement_and_each_if_with_its_true_branch() {
    let source = "let n: Int = read()\nif n > 0:\n    print(1)\nelse:\n    print(2)\nprint(n)\n";
    let program = built(source);
    let mut lines: Vec<usize> = (program.statements().iter())
        .map(|span| line(source, span.start))
        .collect();
    lines.dedup();
    assert_eq!(lines, [1, 2, 3, 5, 6]);
    let branches: Vec<(usize, usize)> = (program.branches().iter())
        .map(|branch| {
            (
                line(source, branch.test.start),
                line(source, branch.taken.start),
            )
        })
        .collect();
    assert_eq!(branches, [(2, 3)]);
}

#[test]
fn a_plain_program_has_no_grader_no_cpp_body_and_nothing_written_back() {
    let program = built("print(1)\n");
    assert_eq!(program.grader_entry(), None);
    assert!(!program.cpp_only());
    assert!(program.written_back().is_empty());
}

#[test]
fn a_cpp_body_runs_only_as_cpp() {
    let program = built("fn twice(x: Int) -> Int = @cpp(\"return x * 2;\")\nprint(twice(2))\n");
    assert!(program.cpp_only());
}

#[test]
fn a_judge_is_a_decorated_fn_not_a_mention() {
    for name in cpl_facade::JUDGE_DECORATORS {
        let source = format!("@{name}\nfn f():\n    print(1)\n");
        assert!(cpl_facade::has_judge(&source), "@{name}");
    }
    assert!(!cpl_facade::has_judge("# @alt\nprint(\"@check\")\n"));
}

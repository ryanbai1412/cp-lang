//! Where a declaration's errors are said: at once for the program's, where the program
//! reaches it for a library's (§11.6).

use super::*;

/// A program's declaration errors are said in this order, each once, wherever the pipeline
/// keeps them until it says them.
#[test]
fn a_programs_declaration_errors_keep_their_order() {
    let src = "struct S<T = Nope>:\n    x: Int\ntype A = A\nimpl Missing for Int:\n    fn f(self) -> Int = 1\nstruct P:\n    y: Unknown2\n@stress(perm)\ntype Q = Int\nstruct P:\n    z: Int\nprint(1)\n";
    let said: Vec<String> = diags(src).iter().map(|d| d.to_string()).collect();
    assert_eq!(
        said,
        [
            "decls: program:10:1: E0202: a second struct `P` with 0 type parameters (R02.11.4)",
            "decls: E0204: unknown type `Nope`",
            "decls: E0204: unknown type `Unknown2`",
            "decls: program:3:1: E0323: type alias `A` is defined by itself, so it names no type",
            "decls: E0204: unknown trait `Missing`",
        ]
    );
}

/// No library question (§11.6): a library's declaration error is said when the program reaches
/// the item, as std's is, while an inline `mod` is the program's own and says its at once.
#[test]
fn a_librarys_declaration_error_is_said_where_the_program_reaches_it() {
    let std = Std::read();
    let dir = std::env::temp_dir().join(format!("cpl-reach-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib = "pub fn bad(x: Nope) -> Int = 1\n\npub fn ok() -> Int = 1\n";
    std::fs::write(dir.join("kit/broken.cp"), lib).unwrap();
    let roots = Roots::from(&dir);
    let said = |src: &str| match compile(&std, src, &roots, false, None, Checks::CHECKED) {
        Ok(c) => raw(&c).output,
        Err(ds) => ds.iter().map(|d| format!("{d}\n")).collect(),
    };
    assert_eq!(said("use kit.broken.{ok}\nprint(ok())\n"), "1\n");
    let reached = said("use kit.broken.{bad}\nprint(bad(1))\n");
    assert!(reached.contains("unknown type `Nope`"), "{reached}");
    let inline = format!(
        "mod broken:\n{}print(1)\n",
        lib.lines()
            .map(|l| format!("    {l}\n"))
            .collect::<String>()
    );
    let own = said(&inline);
    assert!(own.contains("unknown type `Nope`"), "{own}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A library body that does not check is never accepted: called, its error is said where it
/// is; reached only through a trait, it is refused, the pipeline's to fix (until every std fn
/// checks), never passed over.
#[test]
fn a_library_body_that_does_not_check_is_never_accepted() {
    let std = Std::read();
    let dir = std::env::temp_dir().join(format!("cpl-body-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("kit")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib = "use std.list.{Str}\nuse std.show.{Show}\npub fn bad() -> Int = true\n\npub struct W:\n    x: Int\n\nimpl Show for W:\n    fn show(self) -> Str = 1\n";
    std::fs::write(dir.join("kit/wrong.cp"), lib).unwrap();
    let roots = Roots::from(&dir);
    let said = |src: &str| compile(&std, src, &roots, false, None, Checks::CHECKED);
    let called = said("use kit.wrong.{bad}\nprint(bad())\n");
    let through = said("use kit.wrong.{W}\nprint(W(x: 1))\n");
    std::fs::remove_dir_all(&dir).unwrap();
    let called = called
        .err()
        .expect("a called library body that does not check is accepted");
    assert!(called.iter().any(|d| !d.is_refusal()), "{called:?}");
    let through = through
        .err()
        .expect("a library body reached through a trait is accepted");
    assert!(
        through.first().is_some_and(|d| d.is_refusal()),
        "{:?}",
        through.iter().map(|d| d.to_string()).collect::<Vec<_>>()
    );
}

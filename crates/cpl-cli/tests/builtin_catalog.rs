//! `cpl_check::BUILTIN_MEMBERS` is what editors show for a `List`, a
//! `Str`, a `Map`, a `Set` and an array. Each row is run here, so the
//! catalog cannot say something `cpl run` does not do.

use std::process::Command;

use cpl_check::{prelude_names, MemberKind, PreludeKind, Receiver, BUILTIN_MEMBERS};

/// The values every row's `example` is written against.
const FIXTURE: &str = "var xs = [3, 1, 2]\nvar s = \"hello\"\nvar m = Map<Int, Int>()\nm[1] = 10\nm[5] = 50\n\
                       var st = Set<Int>()\nst.insert(4)\nst.insert(9)\nvar a: [0..3]Int = 0\na[1] = 7\n";

/// Runs `body` after the fixture; the error output when it fails.
fn run(name: &str, body: &str) -> Result<(), String> {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("builtin_catalog");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.cp"));
    std::fs::write(&path, format!("{FIXTURE}{body}\n")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl")).arg("run").arg(&path).output().unwrap();
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

#[test]
fn every_row_of_the_catalog_runs_with_the_type_it_names() {
    let mut wrong = Vec::new();
    for (i, member) in BUILTIN_MEMBERS.iter().enumerate() {
        let who = format!("{}.{}", member.receiver.name(), member.name);
        let written = match member.kind {
            MemberKind::Get => format!(".{}", member.name),
            MemberKind::Method => format!(".{}(", member.name),
        };
        if !member.example.contains(&written) || member.example.contains(&format!(".{}()", member.name)) && member.kind == MemberKind::Get {
            wrong.push(format!("{who}: the example `{}` does not write it as a {:?}", member.example, member.kind));
        }
        if member.ret.is_empty() != member.example_ty.is_empty() {
            wrong.push(format!("{who}: `ret` and `example_ty` disagree on whether it gives a value"));
        }
        let body = if member.example_ty.is_empty() {
            member.example.to_string()
        } else {
            format!("let got: {} = {}\nprint(got)", member.example_ty, member.example)
        };
        if let Err(why) = run(&format!("row{i}"), &body) {
            wrong.push(format!("{who}: `{}` does not run as written:\n{why}", member.example));
        }
        // Built-in `Get` properties are read without parentheses.
        if member.kind == MemberKind::Get {
            let called = format!("let got: {} = {}()\nprint(got)", member.example_ty, member.example);
            match run(&format!("row{i}_called"), &called) {
                Ok(()) => wrong.push(format!(
                    "{who}: `{}()` should be rejected (getters are read without parentheses)",
                    member.example
                )),
                Err(why) if !why.contains("is read without parentheses") => wrong.push(format!(
                    "{who}: `{}()` failed with unexpected error (expected `is read without parentheses`):\n{why}",
                    member.example
                )),
                Err(_) => {}
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_built_in_type_has_rows_and_no_name_twice() {
    for receiver in Receiver::ALL {
        let names: Vec<&str> = cpl_check::builtin_members(receiver).map(|m| m.name).collect();
        assert!(names.contains(&"len"), "{receiver:?} has `len`");
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "{receiver:?} lists a name twice");
    }
    let push = cpl_check::builtin_members(Receiver::List).find(|m| m.name == "push").unwrap();
    assert_eq!(push.signature(), "push(x: T)");
    let first = cpl_check::builtin_members(Receiver::Map).find(|m| m.name == "first").unwrap();
    assert_eq!(first.signature(), "first: ?(K, V)");
}

#[test]
fn the_prelude_names_are_the_ones_a_file_can_use() {
    let names = prelude_names();
    for (name, kind) in [("Int", PreludeKind::Type), ("F64", PreludeKind::Type), ("print", PreludeKind::Value), ("sum", PreludeKind::Value), ("read", PreludeKind::Value)] {
        assert!(names.contains(&(name, kind)), "`{name}` is a {kind:?}");
    }
    assert!(names.iter().any(|(name, kind)| *name == "SegTree" && matches!(kind, PreludeKind::Std(_))));
    for name in ["Interactor", "Subset"] {
        assert!(names.contains(&(name, PreludeKind::Type)), "`{name}` is a type");
    }
    // Only what `stdlib/prelude.cp` lists, which `scripts/check.sh` holds to 80.
    let std = names.iter().filter(|(_, kind)| matches!(kind, PreludeKind::Std(_))).count();
    assert!((1..=80).contains(&std), "{std} names from the prelude");
    assert!(!names.iter().any(|(name, _)| *name == "INF"));
    let mut seen = std::collections::HashSet::new();
    // `List` is a type and, called, a value: once of each kind.
    for (name, kind) in &names {
        assert!(seen.insert((*name, std::mem::discriminant(kind))), "`{name}` is listed twice as a {kind:?}");
    }
}

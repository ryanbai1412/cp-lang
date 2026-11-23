//! `cpl_check::BUILTIN_MEMBERS` is what editors show for a `List`, a
//! `Str`, a `Map`, a `Set` and an array. Each row is run here, so the
//! catalog cannot say something `cpl run` does not do.

use std::path::Path;
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
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&path)
        .output()
        .unwrap();
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
        if !member.example.contains(&written)
            || member.example.contains(&format!(".{}()", member.name))
                && member.kind == MemberKind::Get
        {
            wrong.push(format!(
                "{who}: the example `{}` does not write it as a {:?}",
                member.example, member.kind
            ));
        }
        if member.ret.is_empty() != member.example_ty.is_empty() {
            wrong.push(format!(
                "{who}: `ret` and `example_ty` disagree on whether it gives a value"
            ));
        }
        // `_`: a view or adapter, whose type the program does not write.
        let ann = match member.example_ty {
            "_" => String::new(),
            ty => format!(": {ty}"),
        };
        let body = if member.example_ty.is_empty() {
            member.example.to_string()
        } else if member.example_ty == "Bool" {
            format!("let got{ann} = {}\nprint(yes(got))", member.example)
        } else if member.example_ty.starts_with('?') {
            format!(
                "let got{ann} = {}\nif let _ = got: print(1)\nelse: print(0)",
                member.example
            )
        } else {
            let show = if member.example_ty == "_" { "1" } else { "got" };
            format!("let got{ann} = {}\nprint({show})", member.example)
        };
        if let Err(why) = run(&format!("row{i}"), &body) {
            wrong.push(format!(
                "{who}: `{}` does not run as written:\n{why}",
                member.example
            ));
        }
        // Built-in `Get` properties are read without parentheses.
        if member.kind == MemberKind::Get {
            let called = format!(
                "let got{ann} = {}()\nif let _ = Some(got): print(1)",
                member.example
            );
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
        let names: Vec<&str> = cpl_check::builtin_members(receiver)
            .map(|m| m.name)
            .collect();
        assert!(names.contains(&"len"), "{receiver:?} has `len`");
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "{receiver:?} lists a name twice");
    }
    let push = cpl_check::builtin_members(Receiver::List)
        .find(|m| m.name == "push")
        .unwrap();
    assert_eq!(push.signature(), "push(x: T)");
    let first = cpl_check::builtin_members(Receiver::Map)
        .find(|m| m.name == "first")
        .unwrap();
    assert_eq!(first.signature(), "first: ?(K, V)");
}

#[test]
fn the_prelude_names_are_the_ones_a_file_can_use() {
    let names = prelude_names();
    for (name, kind) in [
        ("Int", PreludeKind::Type),
        ("F64", PreludeKind::Type),
        ("print", PreludeKind::Value),
        ("sum", PreludeKind::Value),
        ("read", PreludeKind::Value),
    ] {
        assert!(names.contains(&(name, kind)), "`{name}` is a {kind:?}");
    }
    assert!(names
        .iter()
        .any(|(name, kind)| *name == "SegTree" && matches!(kind, PreludeKind::Std(_))));
    for name in ["Interactor", "Subset"] {
        assert!(
            names.contains(&(name, PreludeKind::Type)),
            "`{name}` is a type"
        );
    }
    // Only what `std/prelude.cp` lists.
    let std = names
        .iter()
        .filter(|(_, kind)| matches!(kind, PreludeKind::Std(_)))
        .count();
    assert!((1..=250).contains(&std), "{std} names from the prelude");
    assert!(!names.iter().any(|(name, _)| *name == "INF"));
    let mut seen = std::collections::HashSet::new();
    // `List` is a type and, called, a value: once of each kind.
    for (name, kind) in &names {
        assert!(
            seen.insert((*name, std::mem::discriminant(kind))),
            "`{name}` is listed twice as a {kind:?}"
        );
    }
}

#[test]
#[ignore = "spec/conformance/std-seq/strview-runs.cp"]
fn sequence_method_parity_across_list_array_str_and_views() {
    use std::collections::BTreeSet;
    let list_names: BTreeSet<&str> = cpl_check::builtin_members(Receiver::List)
        .map(|m| m.name)
        .collect();
    let str_names: BTreeSet<&str> = cpl_check::builtin_members(Receiver::Str)
        .map(|m| m.name)
        .collect();
    let arr_names: BTreeSet<&str> = cpl_check::builtin_members(Receiver::Array)
        .map(|m| m.name)
        .collect();

    // Every general sequence method on `List` must also exist on `Str`
    // (excluding List-only `to_list`, `extend`, `reserve`, `insert`, `remove`,
    // and numeric-only `sum`/`prod` — `Char` is not numeric).
    let list_only_vs_str: BTreeSet<&str> = [
        "to_list", "extend", "reserve", "insert", "remove", "sum", "prod",
    ]
    .into_iter()
    .collect();
    let missing_on_str: Vec<&&str> = list_names
        .iter()
        .filter(|n| !list_only_vs_str.contains(*n) && !str_names.contains(*n))
        .collect();
    assert!(
        missing_on_str.is_empty(),
        "`Str` is missing List sequence methods: {missing_on_str:?}"
    );

    // Every non-growable sequence method on `List` must also exist on `[D]T`.
    let growable_only: BTreeSet<&str> = [
        "push", "pop", "insert", "remove", "resize", "truncate", "extend", "clear", "reserve",
        "to_list", "copy", "reversed", "rev",
    ]
    .into_iter()
    .collect();
    let missing_on_arr: Vec<&&str> = list_names
        .iter()
        .filter(|n| !growable_only.contains(*n) && !arr_names.contains(*n))
        .collect();
    assert!(
        missing_on_arr.is_empty(),
        "`Array` is missing List sequence methods: {missing_on_arr:?}"
    );

    // Run the full sequence parity program (List, [D]T, Str, ListView, StrView) in both
    // interpreter and compiled C++17 and verify identical output.
    let prog =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs/seq_str_parity.cp");
    let interp = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&prog)
        .output()
        .unwrap();
    assert!(
        interp.status.success(),
        "interpreter failed on seq_str_parity.cp:\n{}",
        String::from_utf8_lossy(&interp.stderr)
    );

    let cpp = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg("--release")
        .arg(&prog)
        .output()
        .unwrap();
    assert!(
        cpp.status.success(),
        "C++ run failed on seq_str_parity.cp:\n{}",
        String::from_utf8_lossy(&cpp.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&interp.stdout),
        String::from_utf8_lossy(&cpp.stdout),
        "interpreter and C++17 disagreed on seq_str_parity.cp"
    );
}

#[test]
fn ordered_container_method_parity_and_range_len_guard() {
    use std::collections::BTreeSet;
    let set_names: BTreeSet<&str> = cpl_check::builtin_members(Receiver::Set)
        .map(|m| m.name)
        .collect();
    let map_names: BTreeSet<&str> = cpl_check::builtin_members(Receiver::Map)
        .map(|m| m.name)
        .collect();

    // Every ordered range/bound primitive shared by `Set` and `Map` must be present on both.
    for required in [
        "first", "last", "first_ge", "first_gt", "last_le", "last_lt", "range", "len", "is_empty",
        "contains", "remove", "clear",
    ] {
        assert!(
            set_names.contains(required),
            "`Set` is missing ordered member `{required}`"
        );
        assert!(
            map_names.contains(required),
            "`Map` is missing ordered member `{required}`"
        );
    }
    assert!(
        set_names.contains("set"),
        "`Set` is missing `set(x, on: Bool)`"
    );
    assert!(
        set_names.contains("drain"),
        "`Set` is missing `drain(after:/before:, while:)`"
    );

    // `SortedSet` and `SortedBag` in `std/bag.cp` must expose the ordered API surface.
    let bag_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std/bag.cp"),
    )
    .unwrap();
    for required in ["fn first_true(", "fn last_true(", "fn drain(", "fn range("] {
        assert!(
            bag_src.contains(required),
            "`std/bag.cp` is missing `{required}`"
        );
    }
    assert!(
        !bag_src.contains("fn first_where("),
        "`std/bag.cp` must not expose removed `first_where`"
    );

    // `Set.range(r).len` and `Map.range(r).len` must fail typecheck with help naming SortedSet/SortedBag.
    for (idx, (label, code)) in [
        ("Set", "var s: Set<Int> = Set()\nprint(s.range(1..5).len)\n"),
        (
            "Map",
            "var m: Map<Int, Int> = Map()\nprint(m.range(1..5).len)\n",
        ),
        (
            "Set.rev",
            "var s: Set<Int> = Set()\nprint(s.range(1..5).rev.len)\n",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let path = std::env::temp_dir().join(format!(
            "cpl_range_len_guard_{}_{idx}.cp",
            std::process::id()
        ));
        std::fs::write(&path, code).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("check")
            .arg(&path)
            .output()
            .unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(
            !out.status.success(),
            "{label}.range(r).len unexpectedly passed `cpl check`"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("SortedSet") && stderr.contains("SortedBag"),
            "{label}.range(r).len error help should name SortedSet and SortedBag, got:\n{stderr}"
        );
    }

    // Run ordered_containers_parity.cp in both interpreter and compiled C++17.
    let prog = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/programs/ordered_containers_parity.cp");
    let expected = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/programs/ordered_containers_parity.out"),
    )
    .unwrap();
    for (mode, args) in [("interp", vec!["run"]), ("cpp", vec!["run", "--release"])] {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(&args)
            .arg(&prog)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{mode} failed on ordered_containers_parity.cp:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            expected,
            "{mode} output mismatch on ordered_containers_parity.cp"
        );
    }
}

/// Every copying twin of an in-place method (`xs.reversed()` beside
/// `xs.reverse()`, `xs.shuffled()` beside std.rand's `rng.shuffle(mut xs)`)
/// gets a help naming the in-place call; that call must run as written.
#[test]
fn every_in_place_help_names_a_call_that_runs() {
    let std_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut stems: Vec<String> = BUILTIN_MEMBERS
        .iter()
        .filter(|m| m.receiver == Receiver::List && m.mutates && m.params.is_empty())
        .map(|m| m.name.to_string())
        .collect();
    for entry in std::fs::read_dir(&std_dir).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
        for line in text.lines() {
            let Some(head) = line.trim_start().strip_prefix("pub fn ") else {
                continue;
            };
            if head.contains("(mut self, mut ") {
                stems.push(head.split(['<', '(']).next().unwrap().to_string());
            }
        }
    }
    let mut wrong = Vec::new();
    let mut seen = 0;
    for (i, stem) in stems.iter().enumerate() {
        for twin in [format!("{stem}d"), format!("{stem}ed")] {
            if BUILTIN_MEMBERS.iter().any(|m| m.name == twin) {
                continue;
            }
            let src = format!("let xs = [3, 1, 2]\nprint(xs.{twin}())\n");
            let Err(why) = run(&format!("twin{i}_{twin}"), &src) else {
                continue;
            };
            let Some(at) = why.find("then `") else {
                continue;
            };
            let call = &why[at + "then `".len()..];
            let call = &call[..call.find('`').unwrap()];
            seen += 1;
            let prog = format!("use std.rand.{{Rng}}\nvar rng = Rng(1)\nvar ys = [3, 1, 2]\n{call}\nprint(ys.len)\n");
            if let Err(why) = run(&format!("twin{i}_{twin}_runs"), &prog) {
                wrong.push(format!(
                    "`xs.{twin}()` suggests `{call}`, which fails:\n{why}"
                ));
            }
        }
    }
    assert!(
        seen >= 2,
        "only {seen} in-place helps found; the twin scan is broken"
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

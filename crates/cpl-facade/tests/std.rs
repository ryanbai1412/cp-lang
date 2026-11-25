use cpl_facade::{Layer, Session, Snapshot, Std};

#[test]
fn a_name_is_found_in_the_module_whose_text_declares_it() {
    let module = Std.module_of("Heap").expect("std declares Heap");
    assert_eq!(module, "heap");
    assert!(Std.names().contains(&("Heap", "heap")));
    let text = Std.text(module).expect("the compiler carries std.heap");
    assert!(text.contains("struct Heap"), "{module}");
    let outlined = Std.outlines().iter().find(|(name, _)| *name == module);
    assert!(outlined.is_some_and(|(_, outline)| outline.contains("Heap")));
    assert_eq!(Std.text("no_such_module"), None);
}

#[test]
fn a_method_is_named_with_the_module_that_gives_it() {
    let methods: Vec<(String, &str)> = Std.methods().collect();
    assert!(!methods.is_empty());
    for (module, name) in &methods {
        let text = Std.text(&module.replace('.', "/"));
        assert!(
            text.is_some_and(|text| text.contains(&format!("fn {name}"))),
            "{module}.{name}"
        );
    }
    assert!(Std.has_method("sort") && Std.has_method("sorted"));
    assert!(!Std.has_method("no_such_method"));
}

#[test]
fn a_file_uses_the_modules_it_names_and_those_they_use() {
    let source = "use std.heap\nlet h: Heap<Int> = Heap()\nprint(h.len)\n";
    let used = Std.used_by("p.cp", source);
    assert!(used.contains(&"heap".to_string()), "{used:?}");
    let mut sorted = used.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(used, sorted);
    assert_eq!(Std.used_by("p.cp", "print(1)\n"), Vec::<String>::new());
}

#[test]
fn giving_a_module_the_build_carries_changes_nothing() {
    let db = Session::default().with_file("p.cp", "print(1)\n");
    assert_eq!(db.diagnostics("p.cp", Layer::Typed), Vec::new());
    let text = Std.text("heap").unwrap();
    let given = db.at(db.snapshot().with_std("heap", text).unwrap());
    assert_eq!(given.snapshot(), db.snapshot());
    assert!(db.snapshot().with_std("no_such_module", "").is_err());
    assert_eq!(given.diagnostics("p.cp", Layer::Typed), Vec::new());
    assert_eq!(given.std_needs("p.cp"), Vec::<String>::new());
    assert_eq!(given.std_needed("p.cp"), Vec::<String>::new());
}

/// contracts/00-snapshot.md: the snapshot is the only input, and equal
/// snapshots give equal answers. A module given to one snapshot is in
/// that snapshot and in no other.
#[test]
fn a_std_module_is_given_to_one_snapshot_and_no_other() {
    let source = "use std.heap\nlet h: Heap<Int> = Heap()\nprint(h.len)\n";
    let bare = Session::open(
        Snapshot::default()
            .with_embedded_std()
            .with_file("p.cp", source),
    );
    let lacking = bare.diagnostics("p.cp", Layer::Typed);
    assert!(
        lacking.iter().any(|d| d
            .message
            .contains("std module `std.heap` is not loaded yet")),
        "{lacking:?}"
    );
    // With `heap`, the modules the prelude's names come from.
    let needs = bare.std_needs("p.cp");
    assert!(needs.contains(&"heap".to_string()), "{needs:?}");
    assert!(bare.snapshot().std_text("heap").is_none());
    let parts = ["std".to_string(), "heap".to_string()];
    assert_eq!(bare.snapshot().std_outline(&parts).unwrap().0, 2);

    let mut given = bare.clone();
    let mut rounds = 0;
    loop {
        let needed = given.std_needed("p.cp");
        if needed.is_empty() {
            break;
        }
        for module in needed {
            let text = Std.text(&module).unwrap();
            given = given.at(given.snapshot().with_std(&module, text).unwrap());
        }
        rounds += 1;
        assert!(rounds < 20);
    }
    assert_ne!(given.snapshot(), bare.snapshot());
    assert_eq!(given.diagnostics("p.cp", Layer::Typed), Vec::new());
    assert_eq!(given.std_needs("p.cp"), Vec::<String>::new());
    assert!(given.snapshot().std_outline(&parts).is_none());

    // The first snapshot, and a session opened on one equal to it, still
    // lack the module: nothing was given to the process.
    assert_eq!(bare.diagnostics("p.cp", Layer::Typed), lacking);
    let again = Session::open(
        Snapshot::default()
            .with_embedded_std()
            .with_file("p.cp", source),
    );
    assert_eq!(again.snapshot(), bare.snapshot());
    assert_eq!(again.diagnostics("p.cp", Layer::Typed), lacking);
    assert_eq!(again.std_needs("p.cp"), needs);

    // Given the same text again, the snapshot is the same snapshot.
    let twice = given.snapshot().with_std("heap", Std.text("heap").unwrap());
    assert_eq!(&twice.unwrap(), given.snapshot());
    // A build that carries all of std has nothing to be given.
    let full = Session::default().with_file("p.cp", source);
    assert_eq!(full.diagnostics("p.cp", Layer::Typed), Vec::new());
}

/// spec/tools/cp-toml.md, `[libs]`: a library is named with the folder it
/// is in, relative to the `cp.toml`, and found from any folder below it.
#[test]
fn a_library_is_found_from_below_the_cp_toml_that_names_it() {
    let root = std::env::temp_dir().join(format!("cpl-facade-libs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("mine")).unwrap();
    std::fs::create_dir_all(root.join("contest/a")).unwrap();
    std::fs::write(root.join("cp.toml"), "[libs]\nmine = \"mine\"\n").unwrap();
    let found = cpl_facade::libraries(&root.join("contest/a"));
    let want = root.join("mine").canonicalize().unwrap();
    assert_eq!(found, [("mine".to_string(), want)]);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_file_uri_names_the_path_it_decodes_to() {
    #[cfg(not(windows))]
    assert_eq!(
        cpl_facade::file_path("file:///tmp/my%20code/a.cp"),
        "/tmp/my code/a.cp"
    );
    assert_eq!(cpl_facade::file_path("a.cp"), "a.cp");
}

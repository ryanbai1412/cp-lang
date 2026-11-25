use cpl_facade::{Layer, Session, Std};

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
    assert_eq!(Std.needed("p.cp", source), Vec::<String>::new());
}

#[test]
fn giving_a_module_the_compiler_has_changes_nothing() {
    let db = Session::default().with_file("p.cp", "print(1)\n");
    assert_eq!(db.diagnostics("p.cp", Layer::Typed), Vec::new());
    let text = Std.text("heap").unwrap();
    assert_eq!(db.give_std("heap", text), Ok(false));
    assert!(db.give_std("no_such_module", "").is_err());
    assert_eq!(db.diagnostics("p.cp", Layer::Typed), Vec::new());
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

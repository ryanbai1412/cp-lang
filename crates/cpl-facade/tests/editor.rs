use cpl_facade::{Definition, Session, Snapshot, World};

fn at(source: &str, needle: &str) -> u32 {
    source.find(needle).unwrap_or_else(|| panic!("{needle}")) as u32
}

/// Contract 9: `definition(pos)` is a function of the snapshot, and its
/// answer is a place in one of the snapshot's files.
#[test]
fn a_name_is_declared_where_its_own_file_or_the_one_it_uses_says() {
    let main = "use std.deque.{Deque}\nlet n = 3\nlet d = Deque<Int>()\nprint(d.len() + n)\n";
    let snapshot = Snapshot::new(World::Closed).with_file("main.cp", main);
    let db = Session::open(snapshot.clone());

    let local = db.definition("main.cp", at(main, "n)"));
    let declared = at(main, "n = 3");
    let span = cpl_diag::Span::new(declared, declared + 1);
    let expected = Definition {
        path: "main.cp".to_string(),
        span,
    };
    assert_eq!(local, Some(expected));

    let used = at(main, "Deque<Int>");
    let imported = db.definition("main.cp", used);
    let imported = imported.unwrap_or_else(|| panic!("Deque has a definition"));
    assert_eq!(imported.path, "std/deque.cp");
    let text = cpl_facade::Std.text("deque").unwrap_or_default();
    let span = imported.span;
    assert_eq!(&text[span.start as usize..span.end as usize], "Deque");
    assert_eq!(
        db.definitions("main.cp", used),
        std::slice::from_ref(&imported)
    );

    // An equal snapshot gives an equal answer; a place with no name gives none.
    let again = Session::open(snapshot);
    assert_eq!(again.definition("main.cp", used), Some(imported));
    assert_eq!(db.definition("main.cp", at(main, "3")), None);
    assert_eq!(db.definitions("main.cp", at(main, "3")), []);
}

/// `type_definition` names the type's declaration and `implementations`
/// each `impl` of it.
#[test]
fn a_values_type_and_its_impls_are_found_from_a_use() {
    let source = "struct Road {\n    len: Int\n}\nimpl Road {\n    fn double(self) -> Int {\n        return self.len * 2\n    }\n}\nlet r = Road { len: 2 }\nprint(r.double())\n";
    let db = Session::default().with_file("p.cp", source);
    let ty = db.type_definition("p.cp", at(source, "r.double"));
    let ty = ty.unwrap_or_else(|| panic!("r has a type"));
    assert_eq!(ty.path, "p.cp");
    assert_eq!(ty.span.start, at(source, "Road {"));
    let impls = db.implementations("p.cp", at(source, "Road {"));
    assert_eq!(impls.len(), 1, "{impls:?}");
    assert!(impls[0].span.start >= at(source, "impl Road"), "{impls:?}");
}

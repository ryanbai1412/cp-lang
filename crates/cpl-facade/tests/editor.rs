use cpl_facade::{Definition, Session, Snapshot, Typed, World};

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
    // The place is in a file the session can show: std is in the snapshot.
    let text = db.file_text("std/deque.cp");
    let text = text.unwrap_or_else(|| panic!("std/deque.cp has a text"));
    assert_eq!(Some(&*text), cpl_facade::Std.text("deque"));
    assert_eq!(db.file_text("std/no_such_module.cp"), None);
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

/// Contract 9: `hover(pos)` gives the place's type, doc and cost, as a
/// function of the snapshot.
#[test]
fn a_place_shows_its_type_its_doc_and_what_kind_of_value_it_is() {
    let main = "/// Twice `x`.\nfn twice(x: Int) -> Int = x * 2\nlet n = 3\nprint(twice(n))\n";
    let snapshot = Snapshot::new(World::Closed).with_file("main.cp", main);
    let db = Session::open(snapshot.clone());

    let used = at(main, "x * 2");
    let value = db.hover("main.cp", used);
    let value = value.unwrap_or_else(|| panic!("x has a hover"));
    assert_eq!(value.span, cpl_diag::Span::new(used, used + 1));
    assert!(value.text.starts_with("x: Int"), "{}", value.text);
    assert_eq!(value.typed, Some(Typed::Int));
    assert!(!value.signature);

    let call = db.hover("main.cp", at(main, "twice(n)"));
    let call = call.unwrap_or_else(|| panic!("twice has a hover"));
    assert!(call.signature, "{}", call.text);
    assert!(
        call.text.starts_with("fn twice(x: Int) -> Int"),
        "{}",
        call.text
    );
    assert_eq!(call.typed, Some(Typed::Fn));
    assert_eq!(call.doc.as_deref(), Some("Twice `x`."));

    // An equal snapshot gives an equal answer; a place with nothing gives none.
    let again = Session::open(snapshot);
    assert_eq!(again.hover("main.cp", used), Some(value));
    assert_eq!(db.hover("main.cp", at(main, "\nlet")), None);
}

//! An edit to one item changes only that item's answers.
use super::*;

/// What a body query reads of an item: its hash.
struct Hashed;
impl Query for Hashed {
    type Key = ItemId;
    type Value = Option<u64>;
    const NAME: &'static str = "hashed";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(db.get::<ItemOf>(id)?.map(|i| i.hash))
    }
}

fn db_with(text: &str) -> (Db, FileId) {
    let db = Db::default();
    let f = FileId::new("main.cp");
    db.set::<Source>(f.clone(), text.into()).unwrap();
    (db, f)
}

fn id(name: &str, nth: u32) -> ItemId {
    ItemId {
        module: "main".into(),
        name: name.into(),
        nth,
        file: FileId::new("main.cp"),
    }
}

const TWO: &str = "fn a() -> Int:\n    return 1\n\nfn b() -> Int:\n    return 2\n";

#[test]
fn items_are_keyed_by_module_name_and_rank() {
    let (db, f) = db_with("fn a():\n    pass\n\nfn a():\n    pass\n\nfn b():\n    pass\n");
    assert_eq!(
        db.get::<Items>(&(f.clone(), "main".into())),
        Ok(vec![id("a", 0), id("a", 1), id("b", 0)])
    );
    assert_eq!(db.get::<ItemAt>(&id("b", 0)), Ok(Some(36)));
    assert_eq!(db.get::<ItemOf>(&id("c", 0)), Ok(None));
}

#[test]
fn an_impl_is_keyed_by_its_head_and_a_statement_block_by_its_rank() {
    let text = "impl Ord for Pair:\n    fn cmp(a: Pair, b: Pair) -> Int = 0\n\nimpl  Pair:\n    fn f(self) -> Int = 0\n\nprint(1)\n\nprint(2)\n";
    let (db, f) = db_with(text);
    let ids = vec![
        id("impl Ord for Pair", 0),
        id("impl Pair", 0),
        id("", 0),
        id("", 1),
    ];
    assert_eq!(db.get::<Items>(&(f.clone(), "main".into())), Ok(ids));
}

#[test]
fn a_body_edit_reanswers_only_that_items_readers() {
    let (db, f) = db_with(TWO);
    let (a, b) = (id("a", 0), id("b", 0));
    let before = (db.get::<Hashed>(&a).unwrap(), db.get::<Hashed>(&b).unwrap());
    db.set::<Source>(f, TWO.replace("return 2", "return 3").into())
        .unwrap();
    let after = (db.get::<Hashed>(&a).unwrap(), db.get::<Hashed>(&b).unwrap());
    assert_eq!(before.0, after.0);
    assert_ne!(before.1, after.1);
    assert_eq!(db.computed()["hashed"], 3, "only b's reader runs again");
}

#[test]
fn a_line_added_above_moves_an_item_without_changing_it() {
    let (db, f) = db_with(TWO);
    let b = id("b", 0);
    let (item, at) = (db.get::<ItemOf>(&b), db.get::<ItemAt>(&b));
    db.set::<Source>(f, format!("# note\n{TWO}").into())
        .unwrap();
    assert_eq!(db.get::<ItemOf>(&b), item);
    assert_eq!(db.get::<ItemAt>(&b), Ok(at.unwrap().map(|k| k + 7)));
    db.get::<Hashed>(&b).unwrap();
    assert_eq!(db.computed()["hashed"], 1);
}

#[test]
fn consts_contexts_operators_and_inline_mod_items_are_keyed_by_name() {
    let text = "const N = 3\n\ncontext io:\n    x: Int\n\nfn (a: P) + (b: P) -> P = a\n\nmod m:\n    fn g() -> Int = 1\n";
    let (db, f) = db_with(text);
    let ids = vec![
        id("N", 0),
        id("io", 0),
        id("+", 0),
        id("m", 0),
        id("m.g", 0),
    ];
    assert_eq!(db.get::<Items>(&(f.clone(), "main".into())), Ok(ids));
    let g = db.get::<ItemOf>(&id("m.g", 0)).unwrap().unwrap();
    assert_eq!(
        SyntaxNode::new_root(g.green).text().trim(),
        "fn g() -> Int = 1"
    );
}

#[test]
fn a_site_is_found_from_its_item_and_moves_only_its_position() {
    let (db, f) = db_with(TWO);
    let span = cpl_diag::Span { start: 2, end: 5 };
    let site = SourceId {
        item: id("b", 0),
        span,
    };
    let at = db.get::<ItemAt>(&site.item).unwrap().unwrap();
    let found = |k: u32| {
        Ok(Some(Definition {
            path: "main.cp".into(),
            span: cpl_diag::Span {
                start: k + 2,
                end: k + 5,
            },
        }))
    };
    assert_eq!(db.get::<SourceOf>(&site), found(at));
    db.set::<Source>(f, format!("# note\n{TWO}").into())
        .unwrap();
    assert_eq!(db.get::<SourceOf>(&site), found(at + 7));
    let gone = SourceId {
        item: id("c", 0),
        span,
    };
    assert_eq!(db.get::<SourceOf>(&gone), Ok(None));
}

#[test]
fn two_programs_main_are_two_modules_of_items() {
    let (db, f) = db_with(TWO);
    let g = FileId::new("other.cp");
    db.set::<Source>(g.clone(), "fn b() -> Int:\n    return 9\n".into())
        .unwrap();
    let b_of = |file: &FileId| ItemId {
        file: file.clone(),
        ..id("b", 0)
    };
    let (in_f, in_g) = (db.get::<ItemOf>(&b_of(&f)), db.get::<ItemOf>(&b_of(&g)));
    assert_ne!(b_of(&f), b_of(&g));
    assert_ne!(in_f, in_g);
    assert_eq!(db.get::<ItemAt>(&b_of(&g)), Ok(Some(0)));
}

#[test]
fn a_library_module_is_the_file_its_root_names_for_it() {
    let db = Db::default();
    let root = FileId::new("a/main.cp");
    let libs: Arc<[(Arc<str>, PathBuf)]> = Arc::from(vec![("kit".into(), PathBuf::from("lib"))]);
    db.set::<Libraries>(root.clone(), libs).unwrap();
    let file = |m: &str| db.get::<ModuleFile>(&(root.clone(), m.into()));
    assert_eq!(
        file("kit.graph.dsu"),
        Ok(Some(FileId::new("lib/graph/dsu.cp")))
    );
    assert_eq!(file("other.m"), Ok(None));
    assert_eq!(file("kit"), Ok(None));
}

#[test]
fn a_file_is_cut_once_however_many_of_its_items_are_read() {
    let (db, f) = db_with(TWO);
    for id in db.get::<Items>(&(f, "main".into())).unwrap() {
        db.get::<ItemOf>(&id).unwrap();
        db.get::<ItemAt>(&id).unwrap();
    }
    assert_eq!(db.computed().get(ItemNodes::NAME), Some(&1));
}

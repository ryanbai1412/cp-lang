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
    db.set::<ModulePath>(f.clone(), "main".into()).unwrap();
    db.set::<ModuleFile>("main".into(), f.clone()).unwrap();
    (db, f)
}

fn id(name: &str, nth: u32) -> ItemId {
    ItemId {
        module: "main".into(),
        name: name.into(),
        nth,
    }
}

const TWO: &str = "fn a() -> Int:\n    return 1\n\nfn b() -> Int:\n    return 2\n";

#[test]
fn items_are_keyed_by_module_name_and_rank() {
    let (db, f) = db_with("fn a():\n    pass\n\nfn a():\n    pass\n\nfn b():\n    pass\n");
    assert_eq!(
        db.get::<Items>(&f),
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
    assert_eq!(db.get::<Items>(&f), Ok(ids));
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

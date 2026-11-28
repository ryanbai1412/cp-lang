use super::*;
use cpl_next_db::items::Items;

fn items(s: &Session, file: &str) -> usize {
    let of = (FileId::new(file), "main".into());
    s.db.get::<Items>(&of).map_or(0, |v| v.len())
}

/// A session moved to a new snapshot reparses the file whose text changed, and nothing else.
#[test]
fn a_session_at_a_new_snapshot_reparses_only_what_changed() {
    let one = (Snapshot::new(World::Closed))
        .with_file("a.cp", "fn f() -> Int:\n    return 1\n")
        .with_file("b.cp", "fn g() -> Int:\n    return 2\n");
    let s = Session::open(one.clone());
    assert_eq!((items(&s, "a.cp"), items(&s, "b.cp")), (1, 1));
    let two = one.with_file(
        "b.cp",
        "fn g() -> Int:\n    return 3\n\nfn h() -> Int:\n    return 4\n",
    );
    let s = s.at(two.clone());
    assert_eq!((items(&s, "a.cp"), items(&s, "b.cp")), (1, 2));
    assert_eq!(s.db.computed().get("parse"), Some(&3));
    let s = s.at(two.without_file(&FileId::new("a.cp")));
    assert_eq!((items(&s, "a.cp"), items(&s, "b.cp")), (0, 2));
    assert_eq!(s.db.computed().get("parse"), Some(&4));
}

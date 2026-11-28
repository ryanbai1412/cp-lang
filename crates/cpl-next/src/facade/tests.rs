use super::*;
use cpl_next_db::db::Query;
use cpl_next_db::items::{Items, Parse};

fn items(s: &Session, file: &str) -> usize {
    let file = FileId::new(file);
    let w = s.work(&file).unwrap();
    (w.db.get::<Items>(&(file, modules::PROGRAM.into()))).map_or(0, |v| v.len())
}

fn parsed(s: &Session) -> usize {
    let w = s.work.lock().unwrap();
    w.db.computed().get(Parse::NAME).copied().unwrap_or(0)
}

/// A session at a new snapshot reparses the file whose text changed, and nothing else; the
/// session it came from still answers for its own snapshot.
#[test]
fn a_session_at_a_new_snapshot_reparses_only_what_changed() {
    let one = (Snapshot::new(World::Closed))
        .with_file("a.cp", "fn f() -> Int:\n    return 1\n")
        .with_file("b.cp", "fn g() -> Int:\n    return 2\n");
    let s1 = Session::open(one.clone());
    assert_eq!((items(&s1, "a.cp"), items(&s1, "b.cp")), (1, 1));
    let two = one.with_file(
        "b.cp",
        "fn g() -> Int:\n    return 3\n\nfn h() -> Int:\n    return 4\n",
    );
    let s2 = s1.at(two.clone());
    assert_eq!((items(&s2, "a.cp"), items(&s2, "b.cp")), (1, 2));
    assert_eq!(parsed(&s2), 3);
    assert_eq!(items(&s1, "b.cp"), 1);
    let s3 = s2.at(two.without_file(&FileId::new("a.cp")));
    assert_eq!((items(&s3, "a.cp"), items(&s3, "b.cp")), (0, 2));
}

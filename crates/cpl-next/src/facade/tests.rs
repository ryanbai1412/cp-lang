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
    let before = parsed(&s1);
    let two = one.with_file(
        "b.cp",
        "fn g() -> Int:\n    return 3\n\nfn h() -> Int:\n    return 4\n",
    );
    let s2 = s1.at(two.clone());
    assert_eq!((items(&s2, "a.cp"), items(&s2, "b.cp")), (1, 2));
    assert_eq!(parsed(&s2) - before, 1);
    assert_eq!(items(&s1, "b.cp"), 1);
    let s3 = s2.at(two.without_file(&FileId::new("a.cp")));
    assert_eq!((items(&s3, "a.cp"), items(&s3, "b.cp")), (0, 2));
}

/// Each file under `from`, copied to the same place under `to`.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (p, q) = (e.path(), to.join(e.file_name()));
        match p.is_dir() {
            true => copy_dir(&p, &q),
            false => drop(std::fs::copy(&p, &q).unwrap()),
        }
    }
}

/// Std is read from its directory when a snapshot is made, into the session's database: a std
/// edit is seen with no rebuild, and only the edited std file is parsed again.
#[test]
fn a_std_edit_is_seen_with_no_rebuild() {
    let dir = std::env::temp_dir().join(format!("cpl-std-edit-{}", std::process::id()));
    copy_dir(Path::new(modules::STD_DIR), &dir);
    let probe = dir.join("probe.cp");
    std::fs::write(&probe, "pub fn probe() -> Int = 41\n").unwrap();
    let src = "use std.probe.{probe}\nprint(probe() + 1)\n";
    let snapshot = || (Snapshot::new(World::Closed).with_std_dir(&dir)).with_file("p.cp", src);
    let run = |s: &Session| s.program("p.cp").map(|p| p.run("").output);
    let s1 = Session::open(snapshot());
    assert_eq!(run(&s1).ok().as_deref(), Some("42\n"));
    let before = parsed(&s1);
    std::fs::write(&probe, "pub fn probe() -> Int = 1\n").unwrap();
    let s2 = s1.at(snapshot());
    assert_eq!(run(&s2).ok().as_deref(), Some("2\n"));
    assert_eq!(parsed(&s2) - before, 1);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// What a program's diagnostics say with std copied to a temp directory and changed by `edit`.
fn said_with_std(name: &str, edit: impl Fn(&Path), src: &str) -> Vec<(Option<String>, String)> {
    let dir = std::env::temp_dir().join(format!("cpl-{name}-{}", std::process::id()));
    copy_dir(Path::new(modules::STD_DIR), &dir);
    edit(&dir);
    let s = Session::open((Snapshot::new(World::Closed).with_std_dir(&dir)).with_file("p.cp", src));
    let said = (s.diagnostics("p.cp", Layer::Typed).into_iter())
        .map(|d| (d.code().map(str::to_string), d.message))
        .collect();
    std::fs::remove_dir_all(&dir).unwrap();
    said
}

/// The record role is filled by the module it marks, once (R11.6): a second marked module is
/// E0202 naming the first.
#[test]
fn a_second_record_module_is_a_duplicate() {
    let edit = |dir: &Path| {
        let text = "@role(record)\nuse std.ops.{Eq}\npub fn two() -> Int = 2\n";
        std::fs::write(dir.join("records.cp"), text).unwrap();
    };
    let said = said_with_std(
        "record-twice",
        edit,
        "use std.records.{two}\nprint(two())\n",
    );
    let dup = "a second module filling `@role(record)`: `std.records` fills it already";
    assert!(
        said.iter()
            .any(|(c, m)| c.as_deref() == Some("E0202") && m == dup),
        "{said:?}"
    );
}

/// A record written where no module fills the record role is E0201 naming the role (R11.6).
#[test]
fn a_record_with_no_record_module_is_unresolved() {
    let edit = |dir: &Path| {
        let p = dir.join("tuple.cp");
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replacen("@role(record)\n", "", 1)).unwrap();
    };
    let said = said_with_std("record-none", edit, "let r = {a: 1}\nprint(r.a)\n");
    let none = "a record shape needs the module `@role(record)` marks, and none does";
    assert!(
        said.iter()
            .any(|(c, m)| c.as_deref() == Some("E0201") && m == none),
        "{said:?}"
    );
}

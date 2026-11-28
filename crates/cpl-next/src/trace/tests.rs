//! A trace is one run over the db's build: calls, places and steps from its event stream.
use super::*;
use crate::linked::set_std;
use cpl_contract::FileId;
use cpl_next_db::items::Source;
use cpl_next_decls::modules;
use cpl_next_ir::build::Checks;

const SRC: &str = "var total = 0\nfn add(x: Int) -> Int = x + 1\nfn bump(k: Int):\n    \
total = total + add(k)\nlet n: Int = read()\nfor i in 0..n:\n    bump(i)\nprint(total)\n";

fn db_with(text: &str) -> (Db, Job) {
    let db = Db::default();
    let f = FileId::new("main.cp");
    db.set::<Source>(f.clone(), text.into()).unwrap();
    set_std(
        &db,
        &modules::std_texts(std::path::Path::new(modules::STD_DIR)),
    )
    .unwrap();
    let job = Job {
        root: f,
        events: false,
        bodies: false,
        judge: None,
        checks: Checks::CHECKED,
    };
    (db, job)
}

#[test]
fn a_trace_reads_its_calls_and_places_from_the_stream() {
    let (db, job) = db_with(SRC);
    let r = trace(&db, &job, "2").unwrap_or_else(|ds| panic!("{ds:?}"));
    assert_eq!(r.outcome().output, "3\n");
    assert!(r.outcome().trap.is_none());
    let calls: Vec<_> = (r.calls().iter())
        .map(|c| {
            (
                c.name.as_ref(),
                c.ordinal,
                c.depth,
                c.args.clone(),
                c.returned.clone(),
            )
        })
        .collect();
    let one = |s: &str| vec![String::from(s)];
    assert_eq!(
        calls,
        [
            ("bump", 1, 1, one("0"), None),
            ("add", 1, 2, one("0"), Some("1".into())),
            ("bump", 2, 1, one("1"), None),
            ("add", 2, 2, one("1"), Some("2".into())),
        ]
    );
    let add = &r.calls()[1];
    let exit = add.exit.expect("add returns");
    assert!(add.entry < exit && exit < r.calls()[0].exit.unwrap());
    let step = r.step(add.entry).unwrap();
    assert_eq!(
        (step.kind, step.func, step.depth),
        (&EventKind::Enter, "add", 2)
    );
    let place = r.place(add.entry).unwrap();
    assert_eq!((place.site.item.name.as_ref(), place.nth), ("add", 0));
    assert_eq!(place.call.len(), 2);
    let writes = (0..r.len())
        .filter_map(|i| r.step(i))
        .filter(|s| matches!(s.kind, EventKind::Write { shown, .. } if shown.as_ref() == "total"));
    assert_eq!(
        writes.map(|s| s.vals.to_vec()).collect::<Vec<_>>(),
        [one("0"), one("1"), one("3")]
    );
    assert!(r.step(r.len()).is_none() && r.place(r.len()).is_none());
}

#[test]
fn a_second_trace_reuses_the_build() {
    let (db, job) = db_with(SRC);
    let (two, three) = (
        trace(&db, &job, "2").unwrap(),
        trace(&db, &job, "3").unwrap(),
    );
    assert_eq!(db.computed()["program"], 1, "one build serves both runs");
    assert_eq!((two.calls().len(), three.calls().len()), (4, 6));
    assert_eq!(three.outcome().output, "6\n");
}

#[test]
fn a_program_that_does_not_build_gives_its_diagnostics() {
    let (db, job) = db_with("let x: Int = \"no\"\n");
    let ds = trace(&db, &job, "").err().expect("does not build");
    assert!(!ds.is_empty());
}

#[test]
fn a_trace_document_types_its_values_and_keeps_the_watched_writes() {
    let src = "var b = false\nvar o: ?Int = None\nvar n = 0\nfor i in 0..2:\n    b = i == 1\n    o = Some(i)\n    n += i\nprint(n)\n";
    let (db, job) = db_with(src);
    let r = trace(&db, &job, "").unwrap_or_else(|ds| panic!("{ds:?}"));
    let all = r.json("main.cp", src, &[]);
    for has in [
        "\"b\":false",
        "\"b\":true",
        "\"o\":null",
        "\"o\":1",
        "\"label\":\"o = Some(i)\"",
        "\"line\":6",
    ] {
        assert!(all.contains(has), "{has} in {all}");
    }
    let watched = r.json("main.cp", src, &["o".into()]);
    assert!(
        watched.contains("\"o\":0") && !watched.contains("\"b\":"),
        "{watched}"
    );
    assert!(!watched.contains("\"label\":\"n += i\""), "{watched}");
}

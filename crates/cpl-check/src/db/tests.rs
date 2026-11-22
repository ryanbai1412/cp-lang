use super::*;

struct MapLoader(Vec<(String, String, String)>);

impl Loader for MapLoader {
    fn load(&self, _from: &str, module: &[String]) -> Option<(String, String)> {
        let key = module.join(".");
        self.0
            .iter()
            .find(|(k, _, _)| k == &key)
            .map(|(_, path, text)| (path.clone(), text.clone()))
    }
}

#[test]
fn imported_module_still_checks_when_analyzed_directly() {
    let mut db = Database::with_loader(Box::new(MapLoader(vec![(
        "lib.bad".into(),
        "lib/bad.cp".into(),
        "pub fn f() -> Int:\n    let x: Int = \"oops\"\n    x\n".into(),
    )])));
    db.set_file("main.cp", "use lib.bad.{f}\nprint(f())\n");
    let main = db.analyze("main.cp").unwrap();
    assert!(main.diagnostics.is_empty(), "{:?}", main.diagnostics);
    let direct = db.analyze("lib/bad.cp").unwrap();
    assert!(!direct.diagnostics.is_empty());
}

#[test]
fn an_import_cycle_is_an_error_naming_the_whole_cycle() {
    let module = |name: &str, text: &str| {
        (
            format!("lib.{name}"),
            format!("lib/{name}.cp"),
            text.to_string(),
        )
    };
    let mut db = Database::with_loader(Box::new(MapLoader(vec![
        module("a", "use lib.b.{g}\npub fn f() -> Int: g()\n"),
        module("b", "use lib.c.{h}\npub fn g() -> Int: h()\n"),
        module("c", "use lib.a.{f}\npub fn h() -> Int: 1\n"),
    ])));
    db.set_file("main.cp", "use lib.a.{f}\nprint(f())\n");
    let main = db.analyze("main.cp").unwrap();
    let around = ["lib/a.cp", "lib/b.cp", "lib/c.cp", "lib/a.cp"].map(String::from);
    assert_eq!(db.import_cycles(), [around.to_vec()]);
    let messages = |a: &Analysis| {
        a.diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        messages(&main),
        ["`lib.a` is in an import cycle: a -> b -> c -> a"]
    );
    for file in ["lib/b.cp", "lib/c.cp", "lib/a.cp"] {
        let direct = db.analyze(file).unwrap();
        assert!(direct.has_errors(), "{file}: {:?}", direct.diagnostics);
    }
}

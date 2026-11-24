use super::*;

fn table(source: &str, traps: bool) -> (Program, Table) {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = crate::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let table = Table::of(&program, traps);
    (program, table)
}

fn of<'t>(program: &Program, table: &'t Table, name: &str) -> &'t Effects {
    let f = program.fns.iter().position(|f| f.name == name).expect(name);
    table.effects(f)
}

const SOURCE: &str = concat!(
    "var total = 0\n",
    "fn add(x: Int):\n",
    "    total += x\n",
    "fn add_twice(x: Int):\n",
    "    add(x)\n",
    "    add(x)\n",
    "fn show(x: Int):\n",
    "    print(x)\n",
    "fn double(x: Int) -> Int = x * 2\n",
    "fn halve(x: Int, d: Int) -> Int = x / d\n",
    "fn apply(f: Int -> Int, x: Int) -> Int = f(x)\n",
    "let n: 1..=9 = read()\n",
    "add_twice(n)\n",
    "show(double(halve(n, n)))\n",
    "print(apply(y => y + total, n))\n",
);

#[test]
fn a_fn_writes_the_globals_it_and_its_callees_store_to() {
    let (program, table) = table(SOURCE, true);
    for name in ["add", "add_twice"] {
        let e = of(&program, &table, name);
        assert_eq!(e.writes.len(), 1, "{name}");
        assert!(!e.io, "{name}");
    }
    assert!(of(&program, &table, "double").writes.is_empty());
}

#[test]
fn printing_is_io_and_a_pure_fn_neither_traps_nor_calls_unknown_code() {
    let (program, table) = table(SOURCE, true);
    assert!(of(&program, &table, "show").io);
    let double = of(&program, &table, "double");
    assert_eq!(double, &Effects::default());
}

#[test]
fn a_division_traps_only_in_a_build_that_checks() {
    let (program, checked) = table(SOURCE, true);
    assert!(of(&program, &checked, "halve").traps);
    let (program, unchecked) = table(SOURCE, false);
    assert!(!of(&program, &unchecked, "halve").traps);
}

#[test]
fn calling_a_fn_value_is_an_unknown_call() {
    let (program, table) = table(SOURCE, true);
    let apply = of(&program, &table, "apply");
    assert!(apply.traps && apply.unknown_call);
}

#[test]
fn a_closure_has_the_effects_of_its_body() {
    let (program, table) = table(SOURCE, true);
    let reads_global = (0..program.closures.len())
        .map(|c| table.closure_effects(&program, c))
        .find(|e| e.traps)
        .expect("the closure reads `total`");
    assert!(reads_global.writes.is_empty() && !reads_global.io);
}

#[test]
fn a_block_touches_what_its_calls_touch_and_an_unseen_fn_value_is_opaque() {
    let (program, table) = table(SOURCE, true);
    let body = |name: &str| {
        let f = program.fns.iter().position(|f| f.name == name).expect(name);
        table.block_access(&program, Some(f), &|_, _| None, &program.fns[f].body)
    };
    let twice = body("add_twice");
    assert!(twice.writes.iter().any(|&(global, _)| global), "{twice:?}");
    assert!(!twice.io && !twice.opaque, "{twice:?}");
    assert!(body("show").io);
    let apply = body("apply");
    assert!(apply.opaque && apply.conflicts(&twice), "{apply:?}");
    assert!(!body("double").conflicts(&twice));
}

#[test]
fn a_closure_made_counts_as_run_and_its_captures_as_the_outer_variables() {
    let source = concat!(
        "fn apply(f: Int -> Int, x: Int) -> Int = f(x)\n",
        "fn shift(k: Int) -> Int = apply(y => y + k, 1)\n",
        "print(shift(2))\n",
    );
    let (program, table) = table(source, true);
    let f = program
        .fns
        .iter()
        .position(|f| f.name == "shift")
        .expect("shift");
    let t = table.block_access(&program, Some(f), &|_, _| None, &program.fns[f].body);
    assert!(
        t.reads.contains(&(false, 0)),
        "the closure reads `k`: {t:?}"
    );
    assert!(!t.opaque && t.calls.len() == 1, "{t:?}");
}

#[test]
fn code_by_itself_names_its_calls_and_keeps_binds_apart_from_stores() {
    let (program, _) = table(SOURCE, true);
    let f = program
        .fns
        .iter()
        .position(|f| f.name == "add_twice")
        .expect("add_twice");
    let own = Access::of_block(&program.fns[f].body);
    assert_eq!(own.calls.len(), 1, "{own:?}");
    assert!(
        own.writes.is_empty() && own.calls_code() && !own.unnamed(),
        "{own:?}"
    );
    let apply = program
        .fns
        .iter()
        .position(|f| f.name == "apply")
        .expect("apply");
    let own = Access::of_block(&program.fns[apply].body);
    assert!(own.values && own.opaque && own.unnamed(), "{own:?}");
}

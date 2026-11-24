//! What hover says a call costs.

use super::*;

#[test]
fn a_prelude_call_hovers_its_doc_and_the_cost_of_its_shape() {
    let src = "let n: 1..=10 = read()\nfn f(a: Int, xs: List<Int>) -> Int:\n    \
               abs(a) + max(a, n) + min(xs) + isqrt(n)\n";
    let at = |needle: &str| src.find(needle).unwrap() as u32;
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let abs = db.hover("a.cp", at("abs")).unwrap();
    assert_eq!((abs.span.start, abs.span.end), (at("abs"), at("abs") + 3));
    assert_eq!(abs.text, "abs: built-in\ncost: O(1)");
    assert!(abs.doc.unwrap().starts_with("The absolute value"));
    let cost = |db: &mut Database, needle| db.hover("a.cp", at(needle)).unwrap().text;
    assert_eq!(cost(&mut db, "max("), "max: built-in\ncost: O(1)");
    assert_eq!(cost(&mut db, "min("), "min: built-in\ncost: O(n)");
    assert!(cost(&mut db, "isqrt").ends_with("\ncost: O(1)"));
    assert_eq!(cost(&mut db, "read"), "read: built-in\ncost: O(1)");
}

#[test]
fn min_and_max_of_scalars_have_their_type() {
    let src =
        "let (n, k) = read<(1..=9, 0..=99)>()\nlet m = min(n, k)\nlet b = max(n, n)\nprint(m, b)\n";
    assert_eq!(hover(src, "m ="), "m: Int");
    assert!(
        hover(src, "b =").starts_with("b: 1..=9"),
        "{}",
        hover(src, "b =")
    );
}

#[test]
fn a_name_bound_to_read_of_a_type_has_that_type() {
    let src = "let n = read<1..=100>()\nlet xs = read<[n](a: Int, b: Bool)>()\nprint(n, xs.len)\n";
    assert_eq!(hover(src, "n ="), "n: 1..=100\n  storage: 1 B");
    assert!(hover(src, "xs =").starts_with("xs: [n](a: Int, b: Bool)\n  storage: 16 B per element"));
}

#[test]
fn print_and_read_cost_o1_only_for_values_of_a_fixed_size() {
    let src = "fn f(xs: List<Int>, s: Str):\n    let (a, b): (Int, 1..=9) = read()\n    \
               print(a, \"is {b}\")\n    print(xs)\n    print(s)\n    let t: Str = read()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).unwrap().text
    };
    assert_eq!(hover(&mut db, 0, "read"), "read: built-in\ncost: O(1)");
    assert_eq!(hover(&mut db, 0, "print"), "print: built-in\ncost: O(1)");
    assert_eq!(
        hover(&mut db, 1, "print"),
        "print: built-in\ncost: O(xs.len)"
    );
    assert_eq!(hover(&mut db, 2, "print"), "print: built-in");
    assert_eq!(hover(&mut db, 1, "read"), "read: built-in");
}

#[test]
fn ask_conversions_and_rand_cost_o1_for_values_of_a_fixed_size() {
    let src = "fn f(c: Char, s: Str):\n    let r: 0..=1 = ask(\"?\", c)\n    \
               let t: Int = ask(s)\n    print(Int(c) + Int(r == 1) + rand(1, 6) + Int(s))\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    let o1 = |name: &str| Some(format!("{name}: built-in\ncost: O(1)"));
    assert_eq!(hover(&mut db, 0, "ask"), o1("ask"));
    assert_eq!(hover(&mut db, 1, "ask").as_deref(), Some("ask: built-in"));
    assert_eq!(hover(&mut db, 0, "Int("), o1("Int"));
    assert_eq!(hover(&mut db, 1, "Int("), o1("Int"));
    assert_eq!(hover(&mut db, 0, "rand"), o1("rand"));
    assert_eq!(hover(&mut db, 2, "Int("), None);
}

#[test]
fn a_method_cost_names_the_receivers_domain_for_the_structs() {
    let src = "domain V = 0..10\nfn f(bit: Fenwick<V, Int>, v: V) -> Int:\n    \
               bit.prefix(v.l)\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        let text = db.hover("a.cp", at).unwrap().text;
        text.lines()
            .find_map(|l| l.strip_prefix("cost: "))
            .map(str::to_string)
    };
    assert_eq!(cost(&mut db, "prefix(v.l)").as_deref(), Some("O(log |V|)"));
}

#[test]
fn read_with_a_fixed_type_argument_and_a_choice_of_fixed_text_cost_o1() {
    let src = "fn f(ok: Bool) -> Int:\n    print(if ok: \"YES\" else: \"NO\")\n    \
               print(if ok: \"YES\" else: str(1))\n    read<Int>() + read<(Int, Char)>().0\n\
               fn g() -> Str:\n    read<Str>()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(hover(&mut db, 0, "print").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 1, "print"), None);
    assert_eq!(hover(&mut db, 0, "read").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 1, "read").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 2, "read"), None);
}

#[test]
fn integer_domain_positions_and_plain_enums_read_and_print_in_o1() {
    let src = "domain Pos = 1..=10\nenum Axis = X | Y\nenum Shape = Dot | Box(Int)\n\
               fn ask(lo: Pos, hi: Pos) -> Int:\n    print(\"?\", lo, hi)\n    read()\n\
               fn f() -> Axis:\n    let a: Axis = read()\n    print(a)\n    a\n\
               fn g() -> Shape:\n    read()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(cost(&mut db, 0, "print").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 0, "read").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 1, "read").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 1, "print").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 2, "read"), None);
}

#[test]
fn a_number_made_into_a_position_or_a_modular_number_costs_o1() {
    let src = "const MOD = 7\ntype Z = Mod<MOD>\ndomain Node = 0..10\ndomain Cell = Node * Node\n\
               fn f(i: Int, s: Str) -> Int:\n    let a = Node(i)\n    let c = Cell(a, a)\n    let z = Z(i)\n    \
               let w = Z(s.len)\n    a.ord + z.v\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, needle: &str| {
        let at = src.find(needle).unwrap() as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(cost(&mut db, "Node(i)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Cell(a").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Z(i)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Z(s.len)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "domain Node").as_deref(), None);
}

#[test]
fn a_domains_nth_at_and_wrap_and_a_positions_offset_cost_o1() {
    let src = "domain V = 0..10\nfn f(i: Int, v: V) -> Int:\n    let a = V.nth(i)\n    let b = V.at(i)\n    \
               let c = V.wrap(i)\n    let d = v.offset(2)\n    a.ord + b.ord + c.ord + d!.ord\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    assert!(db
        .diagnostics("a.cp")
        .iter()
        .all(|d| d.severity != cpl_diag::Severity::Error));
    let hover = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    assert_eq!(
        hover(&mut db, "nth").as_deref(),
        Some("nth: fn(x: Int) -> V\ncost: O(1)")
    );
    for name in ["at(", "wrap", "offset"] {
        let text = hover(&mut db, name).unwrap();
        assert!(text.ends_with("\ncost: O(1)"), "{name}: {text}");
    }
}

#[test]
fn abs_of_an_integer_is_an_integer_so_a_position_made_of_it_costs_o1() {
    let src = "domain Node = 1..=10\nfn r(i: Int, j: 0..=5) -> Node:\n    let m = min(i, j)\n    let a = abs(max(i, 3))\n    \
               Node(min(m, a))\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    assert!(db
        .diagnostics("a.cp")
        .iter()
        .all(|d| d.severity != cpl_diag::Severity::Error));
    let hover = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    assert_eq!(hover(&mut db, "m =").as_deref(), Some("m: Int"));
    assert_eq!(hover(&mut db, "a =").as_deref(), Some("a: Int"));
    assert!(hover(&mut db, "Node(min")
        .unwrap()
        .ends_with("\ncost: O(1)"));
}

#[test]
fn printing_an_array_or_list_of_fixed_size_values_costs_its_size() {
    let src = "let n: 1..=100 = read()\ndomain V = 1..=n\nvar d: [V]Int = 1\nvar g: [V, V]Char = '.'\nlet xs: List<Int> = read()\nlet ws: List<Str> = read()\n\
               print(d)\nprint(\"?\", d, g)\nlet r: Int = ask(\"?\", g)\nprint(xs)\nprint(ws)\nprint(r)\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let mut hover = |name: &str| {
        db.hover("a.cp", src.find(name).unwrap() as u32)
            .map(|h| h.text)
            .unwrap_or_default()
    };
    assert!(
        hover("print(d)").ends_with("\ncost: O(|V|)"),
        "{}",
        hover("print(d)")
    );
    assert!(hover("print(\"?\", d").ends_with("\ncost: O(|V| + |V| * |V|)"));
    assert!(
        hover("ask(").ends_with("\ncost: O(|V| * |V|)"),
        "{}",
        hover("ask(")
    );
    assert!(
        hover("print(xs)").ends_with("\ncost: O(xs.len)"),
        "{}",
        hover("print(xs)")
    );
    assert!(
        !hover("print(ws)").contains("cost: O("),
        "{}",
        hover("print(ws)")
    );
    assert!(hover("print(r)").ends_with("\ncost: O(1)"));
    let src = "let n: 1..=100 = read()\ndomain V = 1..=n\nfn show(d: [V]Int, g: [V, V]Char):\n    print(d)\n    let r: Int = ask(\"?\", g)\n    print(r)\n";
    db.set_file("b.cp", src);
    let at = src.find("print(d)").unwrap() as u32;
    let costs = db.costs_in("b.cp", at, src.len() as u32);
    let costs: Vec<_> = costs.iter().map(|(_, n, c)| format!("{n} {c}")).collect();
    assert_eq!(costs, ["print O(|V|)", "ask O(|V| * |V|)"]);
}

#[test]
fn checks_with_field_names_types_and_fn_uses() {
    let mut db = Database::new();
    db.set_file(
        "ok.cp",
        "context opt:\n    k: Int = 1\nwith io(base: 1, sep: \",\", precision: 2)\nwith opt(k: 5):\n    let x: Int = opt.k\n    print(x)\nfn f() -> Int uses opt = opt.k\n",
    );
    let diags = db.diagnostics("ok.cp");
    assert!(diags.is_empty(), "{diags:?}");

    db.set_file("bad_field.cp", "with io(nope: 1):\n    print(1)\n");
    let codes_bf: Vec<_> = db
        .diagnostics("bad_field.cp")
        .iter()
        .map(|d| d.code)
        .collect();
    assert!(
        codes_bf.contains(&cpl_diag::codes::UNKNOWN_FIELD),
        "{codes_bf:?}"
    );

    db.set_file("bad_type.cp", "with io(base: 2):\n    print(1)\n");
    let codes_bt: Vec<_> = db
        .diagnostics("bad_type.cp")
        .iter()
        .map(|d| d.code)
        .collect();
    assert!(
        codes_bt.contains(&cpl_diag::codes::TYPE_MISMATCH),
        "{codes_bt:?}"
    );

    db.set_file("bad_uses.cp", "fn f() uses nope:\n    print(1)\n");
    let codes_bu: Vec<_> = db
        .diagnostics("bad_uses.cp")
        .iter()
        .map(|d| d.code)
        .collect();
    assert!(
        codes_bu.contains(&cpl_diag::codes::UNRESOLVED_NAME),
        "{codes_bu:?}"
    );
}

#[test]
fn representation_is_the_data_behind_the_storage_line() {
    let src = "domain V = 0..1000\nlet a: [V](Int, Bool) = read()\nlet o: ?V = None\nprint(a, o)\n";
    let at = |needle: &str| src.find(needle).unwrap() as u32;
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let a = db.representation("a.cp", at("a:")).unwrap();
    assert_eq!(
        (a.bytes, a.max_cells, a.max_bytes),
        (Some(16), Some(1000), Some(16_000))
    );
    assert_eq!((a.field_offsets, a.padding), (vec![0, 8], 7));
    let o = db.representation("a.cp", at("o:")).unwrap();
    assert_eq!((o.bytes, o.none_is_spare), (Some(2), Some(true)));
    assert!(db
        .hover("a.cp", at("o:"))
        .unwrap()
        .text
        .contains("None is a spare value"));
}

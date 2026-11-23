//! Method-form calls, fn values and the inference they drive.

use super::*;

#[test]
fn a_library_fn_in_method_form_hovers_and_types_like_its_free_form() {
    let src = "let s = \"aab\"\nprint(s.runs(by: c => c).count())\nprint(runs(s, by: c => c).count())\nlet a = read<[3]Int>()\nlet k = 1 - a.count(x => x > 0)\nprint(k)\n";
    let method = hover(src, "runs(by");
    assert!(method.starts_with("fn runs<S, T, K>(self: S"), "{method}");
    assert_eq!(method, hover(src, "runs(s"));
    assert_eq!(hover(src, "k = "), "k: Int");
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let an = db.analyze("a.cp").unwrap();
    let call = src.find("s.runs").unwrap() as u32;
    let (item, at) = an.item_at(call).unwrap();
    let shape = item.check.shapes.iter().find(|(span, _)| span.start == at);
    assert_eq!(
        shape.map(|(_, ty)| ty.to_string()).as_deref(),
        Some("Gen<Slice<Str>>")
    );
}

#[test]
fn a_fn_value_whose_param_is_wider_does_not_fit_a_narrower_fn_type() {
    // No subtyping (R02.11.3): `T` into `?T` converts a value, not a fn's parameters.
    let src = "domain P = 0..3\nfn later(a: ?P, b: ?P) -> ?P = b ?? a\nlet g: (?P, P) -> ?P = later\nprint(Int(g(None, P(1))!))\n";
    assert_eq!(codes(src), vec!["E0302"]);
    let wrapped = "domain P = 0..3\nfn later(a: ?P, b: ?P) -> ?P = b ?? a\nlet g: (?P, P) -> ?P = (a, b) => later(a, b)\nprint(Int(g(None, P(1))!))\n";
    assert_eq!(codes(wrapped), Vec::<&str>::new());
}

#[test]
fn a_list_map_lambda_param_takes_the_element_type() {
    let src = "let ys = [1, 2].map(x =>\n    let y = x + 1\n    y\n)\nprint(ys)\n";
    assert_eq!(hover(src, "x =>"), "x: Int");
    assert_eq!(hover(src, "y = x"), "y: Int");
}

#[test]
fn an_impl_method_wins_over_a_library_fn_whose_bound_fails() {
    let src = "use std.algebra.{Reversible}\nstruct Word(s: Str)\nimpl Reversible for Word:\n    fn rev(self) -> Word = Word(s + \"!\")\nprint(Word(\"ab\").rev().s)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_method_call_takes_the_fn_its_receiver_fits_from_any_module() {
    let modules = [
        (
            "lib.sq",
            "pub struct Sq(s: Int)\npub fn area(self: Sq) -> Int = self.s * self.s\n",
        ),
        (
            "lib.ci",
            "pub struct Ci(r: Int)\npub fn area(self: Ci) -> Int = 3 * self.r * self.r\n",
        ),
    ];
    let src = "use lib.sq.*\nuse lib.ci.*\nlet a: Int = Sq(2).area()\nlet b: Int = Ci(1).area()\nprint(a, b)\n";
    let mut db = Database::with_loader(Box::new(Modules(modules.to_vec())));
    db.set_file("a.cp", src);
    assert!(
        db.diagnostics("a.cp").is_empty(),
        "{:?}",
        db.diagnostics("a.cp")
    );
    let analysis = db.analyze("a.cp").unwrap();
    let chosen: Vec<&str> = (analysis.items.iter())
        .flat_map(|item| item.check.resolved.iter())
        .filter(|(_, r)| r.name == "area")
        .map(|(_, r)| r.module.as_str())
        .collect();
    assert_eq!(chosen, ["lib.sq", "lib.ci"]);
}

#[test]
fn a_method_two_modules_give_one_receiver_is_ambiguous() {
    let modules = [
        ("lib.a", "pub fn twice<T>(self: T) -> Int = 2\n"),
        ("lib.b", "pub fn twice<T>(self: T) -> Int = 3\n"),
    ];
    let diags = check_with(&modules, "use lib.a.*\nuse lib.b.*\nprint(5.twice())\n");
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].code, cpl_diag::codes::AMBIGUOUS_METHOD);
    assert!(
        diags[0].message.contains("`lib.a` and `lib.b`"),
        "{}",
        diags[0].message
    );
}

#[test]
fn a_method_no_candidate_takes_says_what_the_free_form_says() {
    let diags = check("let a: ?List<Int> = [1, 2]\nprint(a.sum())\n");
    assert_eq!(
        messages(&diags),
        ["`?List<Int>` does not implement `Seq` required by `sum`"]
    );
    let own =
        check("let a: ?Int = 3\nprint(a.map(x => x + 1) ?? 0)\nprint([1, 2].map(x => x * 2))\n");
    assert!(own.is_empty(), "{own:?}");
}

#[test]
fn inside_if_t_bound_every_t_has_the_bound() {
    let src = "fn plus<T: Add>(a: T, b: T) -> T = a + b\nfn f<T>(o: ?T, d: T) -> ?T:\n    if T: Add:\n        if let x = o: return Some(plus(x, d) + d)\n    o\nprint(f(Some(2), 3)!)\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
    let outside = "fn g<T>(a: T, b: T) -> T:\n    if T: Add:\n        return a + b\n    a + b\n";
    assert_eq!(codes(outside), ["E0606"]);
}

#[test]
fn a_method_form_lambda_body_and_tuple_receiver_method_hover() {
    let src = "domain W = 0..3\nfn covers(xs: List<W>, k: Int) -> Bool = xs.len >= k\nlet both = [false, true].all(d => covers(W.filter(w => (Int(w) == 1) == d).collect(), 1))\nprint(yes(both))\nprint(yes((W, W).all((u, v) => u != v or u == v)))\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert!(
        hover(src, "covers(W").starts_with("fn covers"),
        "{}",
        hover(src, "covers(W")
    );
    assert_eq!(hover(src, "d)"), "d: Bool");
    assert!(
        hover(src, "all((u").contains("all"),
        "{}",
        hover(src, "all((u")
    );
}

#[test]
fn indexing_a_struct_with_an_at_method_calls_it() {
    // R12.4.13: `s[k]` is `s.at(k)`.
    let src = "struct Doubled(pub xs: List<Int>):\n    pub fn at(self, i: Int) -> Int = 2 * xs[i]\nlet d = Doubled([3, 4])\nlet y = d[1]\nprint(y)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert_eq!(hover(src, "y = d"), "y: Int");
}

#[test]
fn a_generic_fn_value_takes_its_params_from_the_fn_param() {
    let src = "struct D2<T>(pub x: T)\nfn d2<T>(x: T) -> D2<T> = D2(x)\nfn ap<A, B>(x: A, by: A -> B) -> B = by(x)\ndomain V = 0..3\nlet y = ap(V.first!, d2)\nprint(Int(y.x), ap(\"a\", d2).x)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert!(
        hover(src, "y = ap").starts_with("y: D2<V>"),
        "{}",
        hover(src, "y = ap")
    );
}

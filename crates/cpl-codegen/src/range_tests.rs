//! A store into a range-typed slot is checked only where the range isn't proven.

use crate::tests::cpp;

#[test]
fn range_typed_stores_emit_check_only_when_unproven() {
    let unchecked_store = cpp("struct S:
    var a: 0..=1_000_000
    b: Int
let k: Int = read()
var s = S(0, 1)
s.a = k
print(s.a)
");
    assert!(
        unchecked_store.contains("cpl::check_range(k, 0, 1000000, \"0..=1_000_000\")"),
        "{unchecked_store}"
    );

    let proven_store = cpp("const P = 998244353
struct M(v: 0..P)
let k: Int = read()
let r: 0..=100 = read()
var x: 0..=1_000_000 = 0
x = r
let m = M(k % P)
print(x, m.v)
");
    assert!(
        !proven_store.contains("cpl::check_range"),
        "proven stores (literal, narrower range var, k % P into 0..P) must elide check_range:\n{proven_store}"
    );
}

#[test]
fn a_let_and_a_branch_on_it_carry_bounds_into_a_range_store() {
    let store = |decl: &str, branch: &str| {
        let cpp = cpp(&format!(
            "struct M(v: 0..10)\nfn f(a: M, b: M) -> M:\n    {decl} r = a.v + b.v - 10\n    M({branch})\nprint(f(M(read()), M(read())).v)\n"
        ));
        let f = &cpp[cpp.find("inline M f(").unwrap()..];
        f[..f.find("\n}").unwrap()].to_string()
    };
    let proven = store("let", "if r < 0: r + 10 else: r");
    assert!(!proven.contains("check_range"), "{proven}");
    let mutable = store("var", "if r < 0: r + 10 else: r");
    assert!(mutable.contains("check_range"), "{mutable}");
    let off_by_one = store("let", "if r <= 0: r + 10 else: r");
    assert!(off_by_one.contains("check_range"), "{off_by_one}");
    let wrong_side = store("let", "if r < 0: r else: r + 10");
    assert!(wrong_side.contains("check_range"), "{wrong_side}");
}

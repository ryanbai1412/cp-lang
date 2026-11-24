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
    // A var stored only at its declaration keeps its bounds; one stored again loses them.
    let unstored = store("var", "if r < 0: r + 10 else: r");
    assert!(!unstored.contains("check_range"), "{unstored}");
    let restored = cpp("struct M(v: 0..10)\nfn f(a: M, b: M) -> M:\n    var r = a.v + b.v - 10\n    if a.v > 3:\n        r = 20\n    M(if r < 0: r + 10 else: r)\nprint(f(M(read()), M(read())).v)\n");
    assert!(restored.contains("cpl::check_range(r < 0"), "{restored}");
    let off_by_one = store("let", "if r <= 0: r + 10 else: r");
    assert!(off_by_one.contains("check_range"), "{off_by_one}");
    let wrong_side = store("let", "if r < 0: r else: r + 10");
    assert!(wrong_side.contains("check_range"), "{wrong_side}");
}

#[test]
fn xor_or_of_non_negative_ranges_stays_under_the_top_bit() {
    let pair = |range: &str| {
        cpp(&format!(
            "struct Arc(id: {range}):\n    fn rev(self) -> Arc = Arc(self.id ^ 1)\nprint(Arc(read()).rev().id)\n"
        ))
    };
    let arc = pair("0..2_147_483_648");
    let rev = &arc[arc.find("Arc_rev(").unwrap()..];
    assert!(
        !rev[..rev.find("\n}").unwrap()].contains("check_range"),
        "{arc}"
    );
    let three = pair("0..3");
    let rev = &three[three.find("Arc_rev(").unwrap()..];
    assert!(
        rev[..rev.find("\n}").unwrap()].contains("check_range"),
        "2 ^ 1 is 3: {three}"
    );
}

#[test]
fn a_position_param_and_a_length_are_below_2_to_31() {
    let out = cpp("let n: Int = read()
domain I = 0..n
fn f(p: I) -> Int: p.ord * 2 + 1
fn g(a: List<Int>) -> Int: a.len * 3 - 1
for p in I: print(f(p))
print(g([1, 2]), I.len * 4)
");
    for check in ["cpl::imul", "cpl::iadd", "cpl::isub"] {
        assert!(!out.contains(check), "{check} left in:\n{out}");
    }
}

#[test]
fn a_local_set_twice_keeps_the_larger_bound() {
    let out = cpp("let k: Int = read()
var x = 5
if k > 0: x = 7
print(x * 3, k * 3)
");
    assert!(!out.contains("cpl::imul(x, 3)"), "{out}");
    assert!(out.contains("cpl::imul(k, 3)"), "{out}");
}

#[test]
fn a_range_return_type_checks_what_it_returns_unless_proven() {
    let unproven = cpp("fn f(x: Int) -> 0..10 = x\nprint(f(read<Int>()))\n");
    assert!(
        unproven.contains("cpl::check_range(x, 0, 9, \"0..10` returned by `f\")"),
        "{unproven}"
    );
    let proven = cpp("fn f(x: 0..5) -> 0..10:\n    if x > 2:\n        return x\n    x + 1\nprint(f(read<Int>()))\n");
    let body = &proven[proven.find("f(ll x)").unwrap()..];
    assert!(
        !body[..body.find("\n}").unwrap()].contains("check_range"),
        "{proven}"
    );
}

#[test]
fn a_store_whose_value_the_facts_bound_skips_its_range_check() {
    let out = cpp("type Len = 0..2_147_483_648
fn take(n: Len) -> Int = n
fn go(p: Len, k: 0..63) -> Int = take(p >> k)
domain V = 0..read<1..=1000>()
var dep: [V]0..=V.count = 0
for v in V:
    dep[v] = 0
let xs: List<Int> = read()
let m: Int = read()
var meta: List<Len> = []
meta.push(xs.len / 4)
meta.push(xs.len >> 3)
meta.push(m / 4)
print(go(5, 1), take(xs.len / 2), meta[0])
for v in V:
    print(dep[v])
");
    // A non-negative value shifted right, a size divided or shifted by a
    // constant, and a 0 under a hi that is never negative are all in range.
    assert!(!out.contains("check_range(p >> k"), "{out}");
    assert!(!out.contains("check_range((ll)xs.size()"), "{out}");
    assert!(!out.contains("check_range(0,"), "{out}");
    // An Int of either sign still is checked.
    assert!(out.contains("cpl::check_range(m >> 2, 0,"), "{out}");
}

#[test]
fn a_guard_on_a_field_bounds_its_reads_until_a_store_or_call() {
    let out = cpp("type Spot = -1..2_147_483_648
struct B:
    var free: Spot
    meta: List<Spot>
fn take(u: Spot) -> Int = u
fn go(b: B) -> Int:
    var u = b.meta.len / 4
    if b.free >= 0:
        u = b.free
    take(u)
fn bump(mut b: B) -> Int:
    var w = 0
    if b.free >= 0:
        b.free = -1
        w = b.free
    take(w)
fn fill(mut b: B):
    for v in [0, -1, 3]:
        b.meta.push(v)
let xs: List<Int> = read()
var b = B(free: -1, meta: [])
fill(mut b)
print(go(b), bump(mut b))
");
    // `u` is a size over 4 or `b.free` where it is >= 0, and a literal's
    // walk stays between its least and greatest cell.
    for v in ["u", "v"] {
        assert!(!out.contains(&format!("check_range({v},")), "{out}");
        assert!(!out.contains(&format!("check_arg({v},")), "{out}");
    }
    // A store to the field ends its guard.
    assert!(out.contains("take(cpl::check_arg(w,"), "{out}");
}

#[test]
fn a_min_with_the_range_end_or_a_max_with_its_start_stores_unchecked() {
    let out = cpp("fn f(n: 1..=100, x: 0..=1000, y: Int) -> Int:
    domain V = 0..n
    var c: 0..=V.count = 0
    c = min(x, V.count)
    var d: 3..=10 = 3
    d = max(min(y, 10), 3)
    var e: 0..=V.count = 0
    e = min(y, V.count)
    c + d + e
print(f(read(), read(), read()))
");
    assert!(out.contains("c = std::min<ll>(x, V_0_len);"), "{out}");
    assert!(
        out.contains("d = std::max<ll>(std::min<ll>(y, 10), 3);"),
        "{out}"
    );
    // `y` may be negative, below the range's start.
    assert!(
        out.contains("e = cpl::check_range(std::min<ll>(y, V_0_len), 0, V_0_len"),
        "{out}"
    );
}

//! Domains, positions, views and the type checks built on them.

use super::*;

#[test]
fn a_name_in_no_scope_and_no_library_says_so_and_offers_the_closest_library_name() {
    let diags = check("print(heap_free_search(3))\n");
    assert_eq!(
        messages(&diags),
        ["no `heap_free_search` in scope, and not in the stdlib (yet)"]
    );
    let name = crate::prelude::std_names()
        .iter()
        .map(|(n, _)| *n)
        .find(|n| {
            n.len() > 5
                && n.chars().all(|c| c.is_ascii_lowercase())
                && crate::check::closest(
                    &format!("{n}x"),
                    crate::prelude::VALUE_NAMES.iter().copied(),
                )
                .is_none()
        })
        .expect("a stdlib function no prelude name is close to");
    let typo = format!("{name}x");
    let diags = check(&format!("print({typo}(1))\n"));
    assert!(
        diags[0].message.ends_with("and not in the stdlib (yet)"),
        "{diags:#?}"
    );
    let help = diags[0].help.as_deref().unwrap_or_default();
    assert!(
        help.starts_with(&format!("did you mean `{name}`, from `std.")),
        "{help}"
    );
    let fix = &diags[0].fixes[0];
    assert_eq!(fix.edits.len(), 2, "rename and import: {fix:#?}");
}

#[test]
fn heterogeneous_operator_traits_check_and_reject_mismatched_rhs() {
    let ok = "\
struct Vec:
    x: F64
    y: F64

impl Add for Vec:
    fn add(a, b) = Vec(a.x + b.x, a.y + b.y)

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k, a.y * k)

impl Div<F64> for Vec:
    fn div(a, k) = Vec(a.x / k, a.y / k)

struct Mat:
    a: F64

impl Mul for Mat:
    fn mul(x, y) = Mat(x.a * y.a)

impl Mul<Vec> for Mat:
    type Out = Vec
    fn mul(lhs, rhs) -> Out = Vec(lhs.a * rhs.x, lhs.a * rhs.y)

let u = Vec(1.0, 2.0)
let v: Vec = (u + u) * 2.0 / 4.0
let m = Mat(3.0)
let mv: Vec = m * v
let m2: Mat = m * m
print(mv.x, m2.a)
";
    let diags = check(ok);
    assert!(diags.is_empty(), "expected no errors, got: {diags:#?}");

    let bad = "\
struct Vec:
    x: F64

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k)

let u = Vec(1.0)
let bad = u * \"nope\"
print(bad.x)
";
    let diags = check(bad);
    assert!(
        diags.iter().any(|d| d.code == cpl_diag::codes::MISSING_IMPL
            && d.message.contains("impl Mul<Str> for Vec")),
        "expected MISSING_IMPL for Vec * Str, got: {diags:#?}"
    );
}

#[test]
fn enumerate_points_to_items() {
    let free = check("let xs = [1, 2]\nfor (i, x) in enumerate(xs): print(i, x)\n");
    assert!(!free.is_empty());
    assert!(
        free[0]
            .help
            .as_deref()
            .unwrap_or_default()
            .contains(".items"),
        "expected `.items` hint on bare `enumerate`: {free:#?}"
    );
    let method = check("let xs = [1, 2]\nfor (i, x) in xs.enumerate(): print(i, x)\n");
    assert!(!method.is_empty());
    assert!(
        method[0].message.contains(".items")
            || method[0]
                .help
                .as_deref()
                .unwrap_or_default()
                .contains(".items"),
        "expected `.items` hint on `.enumerate()`: {method:#?}"
    );
}

#[test]
fn slice_and_reverse_views_are_readonly_and_cannot_outlive_mutated_base() {
    // Mutating the base while a slice or reverse view is still live is E0602.
    assert_eq!(
        codes("var s = \"abcdef\"\nlet v = s[1..4]\ns.push('x')\nprint(v)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var s = \"abcdef\"\nlet r = s.rev\ns.push('x')\nprint(r)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet v = xs[1..3]\nxs.push(5)\nprint(v)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet r = xs.rev\nxs.push(5)\nprint(r)\n"),
        ["E0602"]
    );

    // Mutating through a read-only view is E0401.
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet v = xs[1..3]\nv.push(5)\n"),
        ["E0401"]
    );
    assert_eq!(codes("var xs = [1, 2, 3, 4]\nxs.rev.sort()\n"), ["E0401"]);
    assert_eq!(
        codes("let xs = [1, 2, 3, 4]\nxs[1..3][1] = 9\n"),
        ["E0401", "L0605"]
    );
    assert_eq!(
        codes("var s = \"abcdef\"\ns[1..4][1] = 'x'\n"),
        ["E0401", "L0605"]
    );

    // Returning a view escapes its lending block (E0603); copying explicitly succeeds.
    assert_eq!(codes("fn bad(s: Str) -> Str:\n    s[1..3]\n"), ["E0603"]);
    assert_eq!(
        codes("fn bad(xs: List<Int>) -> List<Int>:\n    xs.rev\n"),
        ["E0603"]
    );
    assert_eq!(
        codes(concat!(
            "fn ok_s(s: Str) -> Str:\n    s[1..3].to_str()\n",
            "fn ok_l(xs: List<Int>) -> List<Int>:\n    xs.rev.to_list()\n",
            "var s = \"abcdef\"\nlet owned = s[1..4].to_str()\ns.push('x')\nprint(owned)\n",
        )),
        Vec::<&str>::new()
    );

    // First-class `.rev` view on a 1D domain array: valid value, E0602 on base mutation, E0603 on escape.
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "struct Tree(at: [Node]Int)\n",
            "let a: [Node]Int = [Node] i => i.ord\n",
            "let t = Tree(a)\n",
            "let post = t.at.rev\n",
            "print(post.len, post.nth(1))\n",
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "var a: [Node]Int = [Node] i => i.ord\n",
            "let v = a.rev\n",
            "a[Node.first!] = 9\n",
            "print(v.nth(1))\n",
        )),
        ["E0602"]
    );
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "fn leak(xs: [Node]Int) -> List<Int>:\n    xs.rev\n",
        )),
        ["E0603"]
    );
}

#[test]
fn domain_subsets_enum_arrays_seq_ops_and_cuts_typecheck() {
    let ok = "\
enum Sign: neg = -1, zero = 0, pos = 1
domain V = 1..=5
domain Item = 1..=5
domain Cut = 0..=5
let edges: List<(u: V, v: V)> = [(V(1), V(2))]
let adj = adjacency(edges, into: Bitset)
let full: Bitset<V> = V.all
let diff = V.all - adj[V(1)]
for v in diff: print(v)
let vals: [Sign]Int = [Sign] s => Int(s)
for s in Sign: print(vals[s])
let (evens, odds) = V.partition(v => Int(v) % 2 == 0)
let w: [V]Int = [V] v => Int(v)
let order = V.sort(by: w)
let groups = V.group(by: v => Int(v) % 2)
let picked = V.first(v => Int(v) > 2)
let p: [Item]Cut = [Item] i => Cut(Int(i) - 1)
let s: [Item]Cut = [Item] i => Cut(Int(i))
let (lo, hi) = minmax(p[Item(1)], s[Item(1)])
let r = lo..hi
var st = SegTree<V.cuts, Int>(+, fill: 0)
let v0 = V(2)
st.set(v0.r, 5)
print(full.len, evens.len, odds.len, order.len, groups.len, picked, r.len, st.fold(..=v0.r))
";
    assert_eq!(check(ok), vec![]);

    let bad_cut = "\
domain Item = 1..=5
domain Cut = 0..=5
let a: [Cut]Int = 0
let _ = a[Item(1)]
";
    assert_eq!(codes(bad_cut), vec![cpl_diag::codes::WRONG_DOMAIN.0]);
}

#[test]
fn destructured_domain_in_scope_for_nested_fns() {
    let ok = "\
fn make(k: Int) -> (domain P, [P]Int):
    domain L = 0..k
    (L, [L] i => i.to_idx)

let (domain C, comp) = make(3)
var total = 0
fn bump(c: C, v: Int):
    total += comp[c] + v

for c in C: bump(c, 1)

fn outer() -> Int:
    let (domain K, xs) = make(4)
    fn pick(k: K) -> Int = xs[k]
    var s = 0
    for k in K: s += pick(k)
    s

print(total, outer())
";
    assert_eq!(check(ok), vec![]);
}

#[test]
fn generic_struct_ctor_unifies_caller_type_params_in_nested_fields() {
    let ok = "\
domain V = 0..3
struct P<V, A>:
    back: [V]?(from: V, via: A)
struct Q<V, A>:
    back: [V]?(V, A)
fn mk_rec<S, M>(s: S, m: M) -> P<S, M>:
    var b: [S]?(from: S, via: M) = None
    b[s] = (from: s, via: m)
    P(b)
fn mk_tup<S, M>(s: S, m: M) -> Q<S, M>:
    var b: [S]?(S, M) = None
    b[s] = (s, m)
    Q(b)
fn mk_one_renamed<V, M>(s: V, m: M) -> P<V, M>:
    var b: [V]?(from: V, via: M) = None
    b[s] = (from: s, via: m)
    P(b)
print(mk_rec(V(1), 2).back[V(1)]!.via, mk_tup(V(1), 3).back[V(1)]!.1, mk_one_renamed(V(1), 4).back[V(1)]!.via)
";
    assert_eq!(check(ok), vec![]);
}

#[test]
fn product_domains_typecheck_hover_and_reject_swapped_factors() {
    let ok = "\
domain V = 0..3
domain K = 1..=2
domain State = (V, K)
var dp: [State]Int = 0
for s in State:
    let (v, k) = s
    dp[s] = v.ord * 10 + k.label
for (v, k) in State:
    dp[(v, k)] += dp[v, k]
let s0 = State(V(1), K(2))
let _ = (s0.0, s0.1)

enum Dir = L | R
domain Named = (v: V, d: Dir, ok: Bool)
var seen: [Named]Int = 0
seen[(V(1), Dir.R, true)] = 42
let ns = Named(v: V(1), d: Dir.R, ok: true)
let _ = (seen[ns], ns.v, ns.d, ns.ok)

fn local_prod(n: Int, m: Int) -> Int:
    domain A = 0..n
    domain B = 0..m
    domain P = (A, B)
    var t: [P]Int = 0
    t[(A(0), B(0))]
";
    assert_eq!(check(ok), vec![]);
    // LSP hover reports product domain definition and combined max size (3 * 2 = 6).
    let state_hover = hover(ok, "State =");
    assert!(
        state_hover.contains("domain State = (V, K)")
            && state_hover.contains("at most 6 positions"),
        "unexpected hover: {state_hover}"
    );

    let bad = "\
domain V = 0..3
domain K = 0..2
domain State = (V, K)
var dp: [State]Int = 0
print(dp[(K(0), V(0))])
print(dp[K(0), V(0)])
print(State(K(0), V(0)))
";
    let errs = check(bad);
    assert!(
        errs.len() >= 3 && errs.iter().all(|d| d.code == cpl_diag::codes::WRONG_DOMAIN),
        "expected E0301 on swapped product factors, got: {errs:?}"
    );
}

#[test]
fn partial_and_total_array_inverse_typecheck() {
    let ok = "\
domain P = 1..=5
domain V = 0..4
enum Dir = N | E | S | W

var a: [P]?V = None
let inv_a: [V]?P = a.inverse()
let _ = inv_a[V(0)]

let n: 1..=5 = 5
var b: [P]?(1..=n) = None
let inv_b: [1..=n]?P = b.inverse()
let _ = inv_b[1]

var c: [P]?Dir = None
let inv_c: [Dir]?P = c.inverse()
let _ = inv_c[Dir.N]

domain I = 0..3
domain J = 1..=3
let perm: [I]J = [J(1), J(2), J(3)]
let inv_perm: [J]I = perm.inverse()
let _ = inv_perm[J(1)]
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
domain P = 1..=5
var xs: [P]?Int = None
let _ = xs.inverse()
";
    assert!(
        !check(bad).is_empty(),
        "expected error for inverse on `[P]?Int` (unbounded Int is not an Index)"
    );
}

#[test]
fn pow_method_is_rejected_and_suggests_star_star() {
    let bad = "\
let a = 2
let b = a.pow(10)
print(b)
";
    let errs = check(bad);
    assert!(
        errs.iter()
            .any(|e| e.message.contains("power uses `**`, not `.pow(...)`")),
        "expected `.pow` error suggesting `**`, got: {errs:?}"
    );
}

#[test]
fn builtin_getters_reject_empty_parentheses_with_fix() {
    let bad = "\
let xs = [1, 2, 3]
let s = \"abc\"
print(xs.len(), xs.is_empty(), xs.first(), xs.last(), xs.rev(), xs.items(), s.len(), s.rev())
";
    let errs = check(bad);
    assert_eq!(errs.len(), 8, "expected 8 getter errors, got: {errs:#?}");
    for e in &errs {
        assert!(
            e.message.contains("is read without parentheses:"),
            "unexpected message: {}",
            e.message
        );
        assert!(!e.fixes.is_empty(), "expected fix-it on: {e:#?}");
    }
    let ok = "\
let xs = [1, 2, 3]
print(xs.len, yes(xs.is_empty), xs.first ?? 0, xs.last ?? 0, xs.rev, xs.items, xs.first(x => x > 1) ?? 0, xs.last(1) ?? 0)
";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );
}

#[test]
fn pos_back_ahead_and_range_drop_take_typecheck() {
    let src = "\
domain D = 1..=5
let p: D = D(3)
let a: ?D = p.ahead(2)
let b: ?D = p.back(1)
let r: Range<D> = D(1)..=D(5)
let r2: Range<D> = r.drop(1).take(2)
let r3: Range<Int> = (0..10).drop(2).take(3)
print(yes(a == None), yes(b == None), r2.len, r3.len)
";
    assert!(
        check(src).is_empty(),
        "expected clean check, got: {:?}",
        check(src)
    );
}
#[test]
fn existential_return_compress_hover_and_escape() {
    let ok = "\
fn compress_vals<D>(xs: [D]Int) -> (domain C, [D]C, [C]Int):
    var uniq: List<Int> = []
    for i in D:
        uniq.push(xs[i])
    uniq.sort()
    uniq = uniq.deduped()
    domain C = 0..uniq.len
    let idx: [D]C = [D] i => C((0..uniq.len).bsearch(k => uniq[k] >= xs[i]))
    let vals: [C]Int = [C] c => uniq[c.ord]
    (C, idx, vals)

domain I = 0..4
let a: [I]Int = [30, 10, 30, 20]
let (domain V, mapped, vals) = compress_vals(a)
print(V.len, mapped[I(0)].ord, vals[mapped[I(3)]])
";
    assert_eq!(check(ok), vec![]);
    let v_hover = hover(ok, "V, mapped");
    assert!(
        v_hover.contains("V"),
        "expected existential domain hover for V, got: {v_hover}"
    );
    let mapped_hover = hover(ok, "mapped,");
    assert!(
        mapped_hover.contains("[I]V"),
        "expected `[I]V` hover for mapped, got: {mapped_hover}"
    );
    let vals_hover = hover(ok, "vals) =");
    assert!(
        vals_hover.contains("[V]Int"),
        "expected `[V]Int` hover for vals, got: {vals_hover}"
    );

    let bad = "\
fn bad_compress<D>(xs: [D]Int) -> [D]Int:
    domain C = 0..2
    let p: [C]Int = 0
    p
";
    let errs = check(bad);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::DOMAIN_ESCAPES),
        "expected E0313 on escaping local domain, got: {errs:?}"
    );
}

#[test]
fn struct_array_field_defaults_and_optional_map_filter_typecheck() {
    let ok = "\
type Z = Mod<998_244_353>
type Vol = 1..=5
struct Barrels:
    count: [Vol]Int = 0
    delta: [0..=4]Z = 0

let b = Barrels()
let _ = (b.count[1], b.delta[0])

domain P = 1..=4
fn is_even(p: P) -> Bool = p.label % 2 == 0
let x: ?P = Some(P(2))
let y: ?Bool = x.map(is_even)
let z: ?Int = x.filter(p => p.label > 1).map(p => p.ord)
let n_opt: ?P = None
let w: ?Int = n_opt.map(p => p.ord)
let _ = (y ?? false, z ?? -1, w ?? -1)
";
    assert_eq!(check(ok), vec![]);
}
#[test]
fn unary_plus_requires_numeric_operand() {
    let ok = "let a = +1\nlet b = +2.5\nprint(a, b)\n";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );

    let bad = "let xs = +[1, 2]\nlet s = +\"hi\"\nprint(xs, s)\n";
    let errs = check(bad);
    assert_eq!(errs.len(), 2, "expected 2 E0302 errors, got: {errs:?}");
    assert!(
        errs.iter()
            .all(|e| e.code == cpl_diag::codes::TYPE_MISMATCH),
        "expected E0302 for unary `+` on non-numeric types, got: {errs:?}"
    );
}

#[test]
fn sum_domain_enum_tuple_record_index_and_collision_diag() {
    let ok = "\
use std.flow.{FlowGraph}
use std.dsu.{Dsu}

domain Row = 0..2
domain Col = 0..3

enum Node:
    Src
    Snk
    Cell(Row, Col)

let _: Int = Node.count
var vals: [Node]Int = 0
for v in Node:
    vals[v] = match v:
        Src => 0
        Snk => 1
        Cell(r, c) => r.ord + c.ord
var g = FlowGraph<Node>()
g.add(Node.Src, Node.Cell(Row(0), Col(0)), 1)
let _ = g.max_flow(Node.Src, Node.Snk)

var dsu = Dsu<Node>()
dsu.unite(Node.Src, Node.Snk)

var fg_tup = FlowGraph<(Row, Col)>()
fg_tup.add((Row(0), Col(0)), (Row(1), Col(2)), 2)
let _ = fg_tup.max_flow((Row(0), Col(0)), (Row(1), Col(2)))

var fg_rec = FlowGraph<(r: Row, c: Col)>()
fg_rec.add((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2)), 3)
let _ = fg_rec.max_flow((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2)))

fn walk_anon() -> Int:
    var total = 0
    for (r, c) in (Row, Col):
        total += r.ord + c.ord
    let xs = [r.ord + c.ord for (r, c) in (Row, Col)]
    total + xs.len
let _ = walk_anon()
";
    assert_eq!(check(ok), vec![]);

    let collision = "\
domain Row = 0..2
domain Col = 0..2
domain Cell = (Row, Col)
enum Node:
    Src
    Cell(Row, Col)
";
    let errs = check(collision);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::DUPLICATE_DEFINITION),
        "expected E0202 at declaration when variant collides with domain, got: {errs:?}"
    );
}
#[test]
fn range_lo_hi_are_rejected_with_start_end_fixes_and_user_fields_stay() {
    let ok = "\
struct Seg(lo: Int, hi: Int)
let s = Seg(1, 5)
let r = 2..=4
print(s.lo, s.hi, r.start, r.end, r.last, r.len)
";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );

    let bad = "let r = 2..=4\nprint(r.lo, r.hi)\n";
    let errs = check(bad);
    assert_eq!(errs.len(), 2, "expected 2 E0503 errors, got: {errs:?}");
    assert!(
        errs.iter()
            .all(|e| e.code == cpl_diag::codes::REMOVED_SYNTAX && !e.fixes.is_empty()),
        "expected E0503 with fixes for Range.lo/.hi, got: {errs:?}"
    );
    assert_eq!(errs[0].fixes[0].edits[0].1, "start");
    assert_eq!(errs[1].fixes[0].edits[0].1, "end");
}
#[test]
fn array_map_preserves_index_domain() {
    let ok = "\
enum Side:
    X
    Y

struct Run(at: Int, end: Int, d: Int):
    fn shift(self, k: Int) -> Run = Run(at + k, end + k, d)

struct Block(seen: [Side]Run, real: [Side]Run):
    fn each(self, f: Run -> Run) -> Block = Block(seen.map(f), real.map(f))

domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => r.ord * 10 + c.ord
let g2: [R, C]Bool = g.map(x => x % 2 == 0)
let nested: [Side][C]Int = [Side] s => [C] c => c.ord
let nested2: [Side][C]Int = nested.map(row => row.map(x => x + 1))
let xs: List<Int> = [1, 2, 3]
let ys: List<Int> = xs.map(x => x * 2)
let zs: List<Int> = xs[0..2].map(x => x + 1)
let cs: List<Int> = \"ab\".map(c => Int(c == 'a'))
let _ = (g2, nested2, ys, zs, cs)
";
    assert_eq!(check(ok), vec![]);

    let wrong_domain = "\
domain A = 0..3
domain B = 0..3
let a: [A]Int = 0
let b: [B]Int = a.map(x => x + 1)
print(b)
";
    let errs = check(wrong_domain);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::TYPE_MISMATCH),
        "expected E0302 when assigning `[A]Int` map result to `[B]Int`, got: {errs:?}"
    );
}

#[test]
fn enum_array_literal_coercion_checks_variant_count() {
    let ok = "\
enum Side = X | Y
struct Run(l: Int, r: Int)
struct Block(seen: [Side]Run)
let a: [Side]Int = [1, 2]
let b = Block(seen: [Run(0, 1), Run(1, 2)])
print(a[X], b.seen[Y].r)
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
enum Side = X | Y
let a: [Side]Int = [1, 2, 3]
print(a[X])
";
    let errs = check(bad);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].code.0, "E0302");
    assert!(
        errs[0].message.contains("holds 2 elements") && errs[0].message.contains("found 3"),
        "{errs:?}"
    );
}

/// The cf2183f dream checks clean: its tree lines (`read<Tree<V>>()`,
/// `t.at.rev`, `t.pos`, `t.below`, `t.sub`) and its 2D prefix sums.
#[test]
#[ignore = "corpus/problems/cf2183f/dream.cp is v0.2 (FillPrefix.sum); ported with the corpus"]
fn dream_cf2183f_checks() {
    let src = include_str!("../../../../corpus/problems/cf2183f/dream.cp");
    let found = check(src);
    assert!(found.is_empty(), "{found:?}");
}

/// A failed `use` keeps the prelude off only the names it lists: the
/// missing name is one error, and what the prelude gives still checks.
#[test]
fn a_failed_import_drops_only_its_names_from_the_prelude() {
    let src = "use std.graph.{no_such}\ndomain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    let found = check(src);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].message.contains("no_such"), "{found:?}");
    let src = "use std.nope.{Rooted}\ndomain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    assert_eq!(check(src).len(), 1, "{:?}", check(src));
}

#[test]
fn mutable_slice_views_lend_and_reject_aliasing() {
    // In-place mutation through a mutable slice of a `var` list or `[D]T` succeeds.
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nxs[1..3].sort()\nxs[1..3][1] = 9\nprint(xs)\n"),
        ["L0605"]
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nv.sort()\nprint(xs)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet a = xs[0..2]\nlet b = xs[2..4]\na.sort()\nb.sort()\nprint(xs)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("domain D = 0..4\nvar a: [D]Int = [D] i => 4 - i.ord\nlet v = a[D(1)..D(3)]\nv.sort()\nprint(a)\n"),
        Vec::<&str>::new()
    );

    // Mutating through a slice of a `let` list or fixed-length mutators are E0401.
    assert_eq!(codes("let xs = [4, 3, 2, 1]\nxs[1..3].sort()\n"), ["E0401"]);
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nxs[1..3].push(5)\n"),
        ["E0401"]
    );

    // Aliasing the parent (write or read) while a mutable slice is live is E0602.
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nxs[2] = 9\nv.sort()\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nprint(xs[2])\nv.sort()\n"),
        ["E0602"]
    );
    assert!(
        codes("var xs = [4, 3, 2, 1]\nlet a = xs[0..3]\nlet b = xs[1..4]\na.sort()\nb.sort()\n")
            .iter()
            .all(|c| *c == "E0602")
            && !codes(
                "var xs = [4, 3, 2, 1]\nlet a = xs[0..3]\nlet b = xs[1..4]\na.sort()\nb.sort()\n"
            )
            .is_empty()
    );
    assert_eq!(
        codes("domain D = 0..4\nvar a: [D]Int = [D] i => 4 - i.ord\nlet v = a[D(1)..D(3)]\na[D(2)] = 0\nv.sort()\n"),
        ["E0602"]
    );
}

#[test]
fn bounds_ranges_and_domain_ranges_check_per_decision_124() {
    assert_eq!(
        codes(
            "domain I = 1..=5\n\
             var s: Set<I> = Set()\n\
             fn walk(c: I, stop: Cut<I>) -> Int = s.range(c.l..stop).count()\n"
        ),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(
            "domain Seg = 0..2\n\
             domain PosB = 0..4\n\
             var piece: [Seg]Range<PosB.cuts> = PosB.cuts.first..PosB.cuts.first\n\
             piece[Seg(0)] = PosB.cuts.first..PosB.cuts.last\n"
        ),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(
            "domain Pos = 1..=5\n\
             fn tally(ls: Range<Pos>) -> Int = ls.len\n\
             let k = tally(Pos)\n"
        ),
        Vec::<&str>::new()
    );
}

#[test]
fn an_int_literal_is_a_position_of_an_integer_domain() {
    let src = "let n: 1..=9 = read()\ndomain V = 0..n\nvar best: [V.cuts]V.cuts = 0\nlet v: V = 0\nbest[0] = 1\nfn f(k: Int):\n    domain K = 0..=k\n    var t: [K]Int = 0\n    t[0] = 1\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_struct_declared_in_a_fn_is_a_type_and_a_constructor_there() {
    let src = "fn f(n: Int) -> Int:\n    struct Acc(sum: Int, count: Int)\n    var best: ?Acc = None\n    best = Acc(n, 1)\n    best!.sum\n";
    assert_eq!(check(src), Vec::new());
}

#[test]
fn inline_modules_extern_fns_and_pattern_params_check() {
    let src = "mod m:\n    extern fn ask(x: Int) -> Int\n    fn swap((a, b): (Int, Int)) -> (Int, Int) = (b, a)\n    fn solve(n: Int) -> Int = swap((ask(n), n)).0\n";
    assert_eq!(check(src), Vec::new());
}

#[test]
fn list_literals_fill_arrays_inside_tuples_and_match_arms() {
    let src = "\
type Site = 0..10
fn pick(k: 0..=1, a: Site, b: Site) -> ([2]Site, [1]Site) = match k:
    0 => ([a, b], [a])
    1 => ([b, a], [b])
";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn leading_index_gives_the_row_view() {
    let src = "domain R = 0..3\ndomain C = 0..4\nvar g: [R, C]Int = 0\nlet row: [C]Int = g[R(1)]\nprint(row[C(2)])\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert_eq!(
        codes("domain R = 0..3\ndomain C = 0..4\nvar g: [R, C]Int = 0\nprint(g[C(1)])\n"),
        ["E0305"]
    );
}

#[test]
fn seq_parameter_takes_anything_walkable() {
    let src = "domain P = 0..5\nfn total(xs: Seq<P>) -> Int:\n    var t = 0\n    for x in xs: t += Int(x.l)\n    t\nprint(total(P.range))\nprint(total([P(1), P(2)]))\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn reversed_range_keeps_its_element_type() {
    let src = "domain P = 0..4\nlet km: 0..=9 = read()\nvar dp: [0..=km, P]Int = 0\nvar row: [P]Int = 0\nfor k in (0..=km).rev:\n    dp[k] = row\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn domain_wrap_gives_a_position() {
    let src = "domain P = 0..5\nvar a: [P]Int = 0\nfor p in P: a[P.wrap(Int(p) - 1)] += 1\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn enums_and_product_domains_take_at() {
    let src = "enum L = A | B | C
domain P = (rows: 0..3, cols: 0..4)
fn f(i: Int) -> Int:
    let l = L.at(i) ?? L.A
    let p = P.at((i, i + 1)) else: return 0
    p.rows + p.cols
";
    assert!(codes(src).is_empty(), "{:?}", check(src));
}

#[test]
fn recursive_let_lambda_checks_and_var_self_ref_is_error() {
    assert_eq!(
        codes("let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)\nprint(fact(5))\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)\nprint(fact(5))\n"),
        ["E0406"]
    );
}

#[test]
fn bounds_literals_and_cut_range_conversion() {
    let ok = "\
domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len
var c: V.cuts = 0
c = 1
var dp: [V.cuts]Int = 0
for i in V.cuts:
    for j in V.cuts.first!..i:
        dp[i] += cost(j..i)
let a: [V]Int = 1
let s = a[V.cuts.first!..V.cuts.last!]
print(c, dp[V.cuts.last!], s.len)
";
    assert_eq!(codes(ok), Vec::<&str>::new());

    // Inclusive cut range `j..=i` must not convert to `Range<V>`.
    let bad_incl = "\
domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len
for i in V.cuts:
    for j in V.cuts.first!..i:
        print(cost(j..=i))
";
    assert_eq!(codes(bad_incl), ["E0302"]);
}

#[test]
fn chained_comparisons_reject_mixed_direction_and_equality() {
    assert_eq!(
        codes("let x = 3\nprint(yes(1 < x <= 5), yes(5 >= x > 1), yes(x == 3), yes(x != 4))\n"),
        Vec::<&str>::new()
    );
    assert_eq!(codes("let x = 3\nprint(1 < x > 2)\n"), ["E0111"]);
    assert_eq!(
        codes("let a = 1, b = 1, c = 1\nprint(yes(a == b == c))\n"),
        ["E0111"]
    );
    assert_eq!(
        codes("let a = 1, b = 2, c = 2\nprint(yes(a < b == c))\n"),
        ["E0111"]
    );
    assert_eq!(
        codes("let a = 1, b = 2, c = 3\nprint(yes(a != b != c))\n"),
        ["E0111"]
    );
}

#[test]
fn moved_from_heap_values_are_rejected() {
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a\nprint(a.len, b.len)\n"),
        ["E0608"]
    );
    assert_eq!(
        codes("let s = \"hi\"\nlet t = s\nprint(s.len, t.len)\n"),
        ["E0608"]
    );
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a.clone()\nprint(a.len, b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a\nprint(b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var a = [1, 2]\nlet b = a\na = [3]\nprint(a.len, b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("let x = 10\nlet y = x\nprint(x, y)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("fn f():\n    let a = [1, 2]\n    let b = a\n    print(a.len, b.len)\nf()\n"),
        ["E0608"]
    );
}

#[test]
fn an_index_of_the_arrays_own_domain_is_proven_by_its_type() {
    let proofs = |src: &str| {
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let mut out = Vec::new();
        for item in analysis.items.iter() {
            for t in &item.check.typed_indexes {
                let access = t.access.shift(item.offset);
                out.push(format!(
                    "{} {}",
                    &src[access.start as usize..access.end as usize],
                    t.proof
                ));
            }
        }
        out
    };
    // Inline, not read from the corpus: ports get rewritten and must not move this test.
    let port = "let n: 1..=3000 = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
@memo
fn lead(l: I, r: I) -> Int:
    if l >= r: return a[l]
    max(a[l] - lead(l + 1, r), a[r] - lead(l, r - 1))
print(lead(I.first!, I.last!))
";
    assert_eq!(
        proofs(port),
        [
            "a[l] l: I, the index space of a",
            "a[l] l: I, the index space of a",
            "a[r] r: I, the index space of a",
        ]
    );
    let src = "let n: 1..=10 = read()
domain V = 1..=n
domain E = 1..n
domain R = 0..3
domain C = 0..4
var g: [R, C]Int = 0
let xs: [V]Int = read()
let ys: [E]Int = read()
for e in E:
    g[R.first!, C.last!] += xs[V.first!] + ys[e]
print(xs[V.first!..=V.last!].sum())
";
    assert_eq!(
        proofs(src),
        [
            "g[R.first!, C.last!] R.first!: R, C.last!: C, the index spaces of g",
            "xs[V.first!] V.first!: V, the index space of xs",
            "ys[e] e: E, the index space of ys",
        ]
    );
    let pairs = "let n: 1..=10 = read()
domain V = 1..=n
domain K = 0..3
domain State = (V, K)
domain R = 0..3
domain C = 0..4
var dp: [State]Int = 0
var g: [R, C]Int = 0
let p: (R, C) = (R.first!, C.last!)
for v in V:
    dp[v, K.first!] = g[p]
print(dp[V.first!, K.last!])
";
    assert_eq!(
        proofs(pairs),
        [
            "dp[v, K.first!] v: V, K.first!: K, the index spaces of dp",
            "g[p] p: (R, C), the index space of g",
            "dp[V.first!, K.last!] V.first!: V, K.last!: K, the index spaces of dp",
        ]
    );
    let wrong = src.replace("ys[e]", "xs[e]");
    assert!(!codes(&wrong).is_empty());
    assert!(!proofs(&wrong).iter().any(|p| p.starts_with("xs[e]")));
}

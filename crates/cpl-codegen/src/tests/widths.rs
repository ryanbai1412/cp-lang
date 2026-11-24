//! How narrow the C++ stores a declared range.

use super::cpp;

#[test]
fn a_struct_field_of_a_range_alias_narrows_like_the_range() {
    let out = cpp("type Sign = -1..=1
struct B:
    var s: Sign
    var t: -1..=1
var b = B(1, -1)
b.s = -b.s
print(b.s, b.t)
");
    assert!(
        out.contains("int8_t s = 0;"),
        "a `Sign` field must take a byte:\n{out}"
    );
    assert!(out.contains("int8_t t = 0;"), "{out}");
}

#[test]
fn a_struct_array_over_a_range_alias_stays_inline() {
    let out = cpp("type Sign = -1..=1
struct T:
    var n: [Sign]Int
var t = T([4, 5, 6])
t.n[-1] += 1
print(t.n[-1])
");
    assert!(
        out.contains("cpl::Row<ll, 3> n;"),
        "`[Sign]Int` has 3 cells:\n{out}"
    );
}

#[test]
fn bool_elements_are_bytes_not_vector_bool() {
    let fixed = cpp("var seen: [10]Bool = false\nseen[3] = true\nprint(yes(seen[3]))\n");
    assert!(
        fixed.contains("std::array<char, 10>") && !fixed.contains("std::vector<bool>"),
        "{fixed}"
    );
    let dyn_len = cpp(
        "let n: Int = read()\nvar seen: [n]Bool = false\nseen[0] = true\nprint(yes(seen[0]))\n",
    );
    assert!(
        dyn_len.contains("std::vector<char>") && !dyn_len.contains("std::vector<bool>"),
        "{dyn_len}"
    );
}

#[test]
fn a_field_ranged_by_a_shifted_const_narrows() {
    let cpp = cpp("const N = 1 << 30\nconst M = N | 1\nstruct Node(l: 0..N, r: 0..M)\nlet a = Node(1, 2)\nprint(a.l + a.r)\n");
    assert!(cpp.contains("uint32_t l = 0;"), "{cpp}");
    assert!(cpp.contains("uint32_t r = 0;"), "{cpp}");
}

#[test]
fn ranged_int_struct_fields_take_the_narrowest_width() {
    let cpp = cpp(concat!(
        "type Z = Mod<998244353>\n",
        "type BigZ = Mod<1_000_000_000_000_000_007>\n",
        "struct Edge(w: -1000..=1000)\n",
        "struct MutBox(x: 0..1000)\n",
        "fn bump(mut a: Int): a += 1\n",
        "let p: Int = read()\n",
        "type DynZ = Mod<p>\n",
        "var b = MutBox(1)\n",
        "bump(mut b.x)\n",
        "print(Z(998244352) * Z(998244352), BigZ(5), Edge(-7).w, DynZ(3), b.x)\n",
    ));
    assert!(
        cpp.contains("uint32_t v = 0;"),
        "Mod<998244353> should narrow to uint32_t: {cpp}"
    );
    assert!(
        cpp.contains("int16_t w = 0;"),
        "-1000..=1000 should narrow to int16_t: {cpp}"
    );
    assert!(
        cpp.contains("ll(a.v)"),
        "narrowed field reads should widen to ll: {cpp}"
    );
    assert!(
        cpp.contains("struct MutBox {\n  ll x = 0;"),
        "field taken by mut reference should stay ll: {cpp}"
    );
    assert!(
        cpp.contains("struct Mod_1000000000000000007_"),
        "wide Mod should stay ll: {cpp}"
    );
}

#[test]
fn a_list_literal_argument_is_built_in_the_params_cells() {
    let out = cpp("fn first(xs: List<-5..=5>) -> Int = xs[0]
let a: 0..=5 = read()
fn neg(v: Int) -> Int = -v
let low: -5..=5 = -a
print(first([-neg(a)]), first([low]))
");
    assert!(
        out.contains("first(const std::vector<int8_t>& xs)"),
        "{out}"
    );
    assert!(!out.contains("std::vector<ll>{"), "{out}");
}

#[test]
fn reserve_keeps_a_narrow_list_narrow() {
    let out = cpp("let k: 1..=9 = read()
var a: List<0..500_000> = []
a.reserve(k)
a.push(3)
print(a.len)
");
    assert!(out.contains("std::vector<uint32_t> a"), "{out}");
}

#[test]
fn items_of_a_slice_keep_its_list_narrow() {
    let out = cpp("var a: List<0..500_000> = [1, 2]\nfor (i, x) in a[..1].items: print(i + x)\nlet w = a[..1]\nprint([w[i] for i in 0..1], w.last!, a.first!)\n");
    assert!(out.contains("std::vector<uint32_t> a"), "{out}");
}

#[test]
fn ranged_array_elements_narrow_to_32_bit_and_keep_it_through_a_param() {
    let top_level = cpp("let n: 1..=100 = read()
domain V = 1..=n
var a: [V]0..=1_000_000 = 0
var b: [V]-1_000_000..=1_000_000 = 0
for v in V:
    a[v] = 500_000
    b[v] = -500_000
var total = 0
for v in V:
    total += a[v] * a[v] + b[v]
print(total)
");
    assert!(
        top_level.contains("std::vector<uint32_t> a"),
        "expected [V]0..=1_000_000 to emit std::vector<uint32_t>:\n{top_level}"
    );
    assert!(
        top_level.contains("std::vector<int32_t> b"),
        "expected [V]-1_000_000..=1_000_000 to emit std::vector<int32_t>:\n{top_level}"
    );
    assert!(
        top_level.contains("ll(a["),
        "expected narrowed array read to widen to ll before arithmetic:\n{top_level}"
    );

    let generic_field = cpp("struct Tree<V>(dep: [V]0..=V.count, far: [V]Int)
fn make<V>() -> Tree<V>:
    var dep: [V]0..=V.count = 0
    var far: [V]Int = 0
    for v in V:
        dep[v] = v.ord
        far[v] = v.ord * 1_000_000_000
    return Tree(dep, far)
let n: 1..=100 = read()
domain V = 1..=n
let t = make<V>()
var ans = 0
for v in V:
    ans += t.dep[v] * t.dep[v] + t.far[v]
print(ans)
");
    assert!(
        generic_field.contains("std::vector<uint32_t> dep;"),
        "expected generic struct field [V]0..=V.count to emit std::vector<uint32_t>:\n{generic_field}"
    );
    assert!(
        generic_field.contains("std::vector<ll> far;"),
        "expected plain [V]Int field to stay std::vector<ll>:\n{generic_field}"
    );

    let widened = cpp("fn sum_all<V>(xs: [V]Int) -> Int:
    var s = 0
    for v in V:
        s += xs[v]
    return s
let n: 1..=100 = read()
domain V = 1..=n
var a: [V]0..=1_000_000 = 0
for v in V:
    a[v] = v.ord
print(sum_all<V>(a))
");
    // A param that only reads takes its caller's narrow storage.
    assert!(
        widened.contains("std::vector<uint32_t> a")
            && widened.contains("(const std::vector<uint32_t>& xs)"),
        "expected array passed to [V]Int param to keep its width in the instance:\n{widened}"
    );
}

#[test]
fn ranged_2d_nested_and_tuple_elements_narrow_through_plain_params() {
    let grid = cpp("let n: 1..=50 = read()
let m: 1..=50 = read()
domain R = 0..n
domain C = 0..m
var g: [R, C]0..=1_000_000 = 0
var s: [R, C]-100..=100 = 0
for r in R:
    for c in C:
        g[r, c] = 500_000
        s[r, c] = -50
var total = 0
for r in R:
    for c in C:
        total += g[r, c] * g[r, c] + s[r, c]
print(total)
");
    assert!(
        grid.contains("cpl::Grid<uint32_t, 2> g"),
        "expected [R, C]0..=1_000_000 to emit cpl::Grid<uint32_t, 2>:\n{grid}"
    );
    assert!(
        grid.contains("cpl::Grid<int8_t, 2> s"),
        "expected [R, C]-100..=100 to emit cpl::Grid<int8_t, 2>:\n{grid}"
    );
    assert!(
        grid.contains("ll(g_row_2["),
        "expected 2D grid read to widen to ll before arithmetic:\n{grid}"
    );

    let nested = cpp("let n: 1..=50 = read()
let m: 1..=50 = read()
domain D = 0..n
domain E = 0..m
var mat: [D][E]0..=1_000_000 = 0
var adj: [D]List<0..=1_000_000> = []
for i in D:
    for j in E:
        mat[i][j] = 500_000
    adj[i].push(500_000)
var ans = 0
for row in mat:
    for x in row:
        ans += x * x
for i in D:
    for x in adj[i]:
        ans += x
print(ans)
");
    assert!(
        nested.contains("std::vector<std::vector<uint32_t>> mat"),
        "expected [D][E]0..=1_000_000 to emit std::vector<std::vector<uint32_t>>:\n{nested}"
    );
    assert!(
        nested.contains("std::vector<std::vector<uint32_t>> adj"),
        "expected [D]List<0..=1_000_000> to emit std::vector<std::vector<uint32_t>>:\n{nested}"
    );

    let tuples = cpp("let n: 1..=50 = read()
domain V = 0..n
var pts: [V](0..=1_000_000, -100..=100) = (0, 0)
for v in V:
    pts[v] = (500_000, -50)
var ans = 0
for v in V:
    let (x, y) = pts[v]
    ans += x * x + y
print(ans)
");
    assert!(
        tuples.contains("std::vector<std::tuple<uint32_t, int8_t>> pts"),
        "expected [V](0..=1_000_000, -100..=100) to emit std::vector<std::tuple<uint32_t, int8_t>>:\n{tuples}"
    );

    let widened = cpp("fn sum_grid<R, C>(g: [R, C]Int) -> Int:
    var s = 0
    for x in g: s += x
    return s
fn sum_row<E>(row: [E]Int) -> Int:
    var s = 0
    for x in row: s += x
    return s
fn sum_pts<V>(pts: [V](Int, Int)) -> Int:
    var s = 0
    for (x, y) in pts: s += x + y
    return s
let n: 1..=20 = read()
domain D = 0..n
var g: [D, D]0..=1_000 = 0
var mat: [D][D]0..=1_000 = 0
var pts: [D](0..=1_000, -100..=100) = (0, 0)
print(sum_grid<D, D>(g) + sum_row<D>(mat[D(0)]) + sum_pts<D>(pts))
");
    for (arg, param) in [
        (
            "cpl::Grid<uint16_t, 2> g;",
            "(const cpl::Grid<uint16_t, 2>& g_2)",
        ),
        (
            "std::vector<std::vector<uint16_t>> mat;",
            "(const std::vector<uint16_t>& row)",
        ),
        (
            "std::vector<std::tuple<uint16_t, int8_t>> pts;",
            "(const std::vector<std::tuple<uint16_t, int8_t>>& pts_2)",
        ),
    ] {
        assert!(
            widened.contains(arg) && widened.contains(param),
            "expected a plain param to take the layout of `{arg}`:\n{widened}"
        );
    }
}

#[test]
fn a_list_built_by_new_stays_narrow_when_called_on_the_type() {
    let out = cpp("struct S:
    pub xs: List<0..=2_000_000_000>

    pub fn new(n: Int) -> S:
        var xs: List<0..=2_000_000_000> = List.filled(n, 0)
        S(xs)
let s = S.new(10)
print(s.xs[9])
");
    assert!(out.contains("std::vector<uint32_t> xs"), "{out}");
}

#[test]
fn an_element_of_a_range_typed_list_stores_back_unchecked() {
    let out = cpp("let n: 1..=100 = read()
var xs: List<0..=1000> = List.filled(n, 0)
var ys: List<0..=1000> = List.filled(n, 0)
for x in xs:
    ys[0] = x
print(ys[0])
");
    assert!(!out.contains("check_range"), "{out}");
}

#[test]
fn an_enum_tag_is_the_smallest_unsigned_type_that_holds_its_variants() {
    let out = cpp("enum Colour: Red | Blue
enum Shape:
    Circle(r: Int)
    Square(s: Int)
let n: 1..=5 = read()
var grid = List.filled(n, Colour.Red)
grid[0] = Colour.Blue
let s = if n > 3: Shape.Circle(n) else: Shape.Square(n)
let area = match s:
    Circle(r): r * r
    Square(x): x * x
print(grid.count(c => c == Colour.Blue), area)
");
    assert!(
        out.contains("enum class Colour : uint8_t { Red, Blue };"),
        "a two-colour grid must take a byte a cell:\n{out}"
    );
    assert!(
        out.contains("enum Tag : uint8_t { Circle, Square }"),
        "{out}"
    );
}

#[test]
fn a_narrow_list_lent_from_a_temporary_owner_is_copied_out_in_its_cells() {
    let out = cpp("struct Box:
    n: Int
    @cache _xs: List<0..1000>
    fn build(self):
        if _xs.is_empty: _xs = List.filled(n, 7)
    @get pub fn items(self) -> List<0..1000>:
        self.build()
        _xs
let n: 1..=100 = read()
print(Box(n, []).items.len)
");
    assert!(out.contains("[&]() -> std::vector<uint16_t>"), "{out}");
    assert!(!out.contains("std::vector<ll>"), "{out}");
}

/// A std seq body run on a narrow list is instantiated at the list's width:
/// `b.adjacent` doesn't widen `b` to `ll` (cf2124c).
#[test]
fn adjacent_on_a_narrow_list_keeps_it_narrow() {
    let out = cpp("let n: 2..=600_000 = read()
let b: [0..n]1..=10 ** 9 = read()
var res = 1
for (x, y) in b.adjacent if y % x != 0:
    res = lcm(res, x / gcd(x, y))
print(res)
");
    assert!(out.contains("std::vector<uint32_t> b"), "{out}");
    assert!(!out.contains("std::vector<ll>"), "{out}");
}

/// A generic fn called with a narrow array takes it at its storage width:
/// the instance's parameter is the caller's `uint32_t` vector, not `ll`.
#[test]
fn a_generic_fn_takes_a_narrow_array_at_its_width() {
    let out = cpp("let n: 1..=100 = read()
let a: [0..n]0..=1_000_000_000 = read()
fn f<A: Positional<Int>>(a: A) -> Int = a.nth(0)
print(f(a))
");
    assert!(out.contains("std::vector<uint32_t> a"), "{out}");
    assert!(out.contains("(const std::vector<uint32_t>& a_"), "{out}");
    assert!(!out.contains("std::vector<ll>"), "{out}");
}

/// Two calls passing different widths share one instance at the wider one;
/// neither caller's value is cut.
#[test]
fn a_generic_fn_called_at_two_widths_takes_the_wider() {
    let out = cpp("let n: 1..=100 = read()
let a: [0..n]0..=1_000_000_000 = read()
let b: [0..n]Int = read()
fn f<A: Positional<Int>>(a: A) -> Int = a.nth(0)
print(f(a), f(b))
");
    assert!(out.contains("std::vector<ll> a"), "{out}");
    assert!(out.contains("std::vector<ll> b"), "{out}");
}

#[test]
fn printing_a_narrow_list_whole_keeps_its_storage() {
    // cf2201b, cf2211d: `print(out)` reads each element; it is no reason to
    // store the list at 64 bits.
    let out = cpp("let n: 1..=300_000 = read()
var out: List<0..=536_870_911> = []
for v in 1..=n: out.push(v)
var grid: [0..n, 0..2]-1..=1 = 0
grid[0, 1] = -1
print(out)
print(grid)
");
    assert!(out.contains("std::vector<uint32_t> out;"), "{out}");
    assert!(out.contains("cpl::Grid<int8_t, 2> grid;"), "{out}");
    assert!(!out.contains("std::vector<ll> out"), "{out}");
}

#[test]
fn a_range_with_a_runtime_bound_stores_at_the_width_its_bound_is_proven_to() {
    // dmopc21c5p4, cf2187f1, coci21c4p4: `n: 1..=1e6` keeps `1..=n`, `0..m`
    // with `m = 2 * n - 1`, and each field of a read edge in 32 bits or less.
    let out = cpp("let n: 2..=1_000_000 = read()
let m = 2 * n - 1
var walk: List<1..=n> = []
walk.push(n)
var kid: [0..m]0..m = 0
kid[1] = m - 1
let roads: [0..n](u: 1..=n, v: 1..=n, w: 1..=1000) = read()
print(walk.len, kid[1], roads[0].w)
");
    assert!(out.contains("std::vector<uint32_t> walk;"), "{out}");
    assert!(out.contains("std::vector<uint32_t> kid;"), "{out}");
    assert!(
        out.contains("std::vector<std::tuple<uint32_t, uint32_t, uint16_t>> roads;"),
        "{out}"
    );
}

#[test]
fn a_read_table_handed_to_a_reading_fn_keeps_its_declared_width() {
    // apio09p1: `oil: [Row, Col]0..=500` given to a fn that only reads it
    // stays a table of 16-bit cells, and so does the fn's parameter.
    let out = cpp(
        "fn total(a: [0..3, 0..3]Int) -> Int = sum(a[i, j] for i in 0..3 for j in 0..3)
let oil: [0..3, 0..3]0..=500 = read()
print(total(oil))
",
    );
    assert!(out.contains("cpl::Grid<uint16_t, 2> oil;"), "{out}");
    assert!(out.contains("(const cpl::Grid<uint16_t, 2>& a)"), "{out}");
}

#[test]
fn a_record_with_a_domain_field_still_stores_its_ranged_fields_narrow() {
    // aac5p3, noi19p1: a `V` field keeps its own type; the ranged ones beside it still narrow,
    // in a read table and in the lists the edges are pushed into.
    let out = cpp("let n: 1..=2e5 = read()
domain V = 1..=n
let hills: [n - 1](u: V, v: V, d: 1..=1e9) = read()
var adj: [V]List<(to: V, d: 1..=1e9)> = []
for (u, v, d) in hills:
    adj[u].push((to: v, d:))
    adj[v].push((to: u, d:))
let given: [0..n](u: V, v: V, s: 0..=1000, e: 0..=1000, late: Bool) = read()
print(adj[V.first!].len, given[0].s)
");
    assert!(
        out.contains("std::vector<std::tuple<int, int, uint32_t>> hills;"),
        "{out}"
    );
    assert!(
        out.contains("std::vector<std::vector<std::tuple<int, uint32_t>>> adj;"),
        "{out}"
    );
    assert!(
        out.contains("std::vector<std::tuple<int, int, uint16_t, uint16_t, bool>> given;"),
        "{out}"
    );
}

#[test]
fn a_ranged_map_stays_narrow_through_params_arrays_swaps_and_literals() {
    let out = cpp("type R = 0..=9
type Key = -1e9..=1e9
let (n, m): (1..=10, 2..=200_000) = read()
var top = Map<Key, R>()
var reach: [0..n]HashMap<Int, 1..=m> = []
fn bump(mut t: Map<Key, R>, k: Key):
    t[k] = 3
fn f(v: Int):
    var here: HashMap<Int, 1..=m> = [v: 1]
    var there: HashMap<Int, 1..=m> = []
    swap(mut there, mut reach[v])
    for (far, hops) in there: here[far] min= hops
    swap(mut reach[v], mut here)
bump(mut top, -5)
for v in 0..n: f(v)
print(top.len, reach[0].len)
");
    for want in [
        "cpl::HashMap<int32_t, uint8_t, cpl::CpHash> top;",
        "bump(cpl::HashMap<int32_t, uint8_t, cpl::CpHash>& t,",
        "std::vector<cpl::HashMap<ll, uint32_t, cpl::CpHash>> reach;",
        "cpl::HashMap<ll, uint32_t, cpl::CpHash> here",
        "cpl::HashMap<ll, uint32_t, cpl::CpHash> there",
    ] {
        assert!(out.contains(want), "expected `{want}`:\n{out}");
    }
    assert!(!out.contains("HashMap<ll, ll"), "{out}");
}

#[test]
fn a_ranged_map_widens_when_it_meets_a_plain_one() {
    let out = cpp("type R = 0..=9
fn fill(mut t: HashMap<Int, Int>):
    t[1] = 1_000_000_000_000
var a: HashMap<Int, R> = []
fill(mut a)
print(a.len)
");
    assert!(
        out.contains("cpl::HashMap<ll, ll, cpl::CpHash> a;"),
        "{out}"
    );
}

#[test]
fn a_comprehension_list_takes_its_elements_inferred_ranges() {
    // A bounded cell and a loop variable over a bounded range each fit in
    // 4 B, so the unannotated list is 8 B a pair, reserved and plainly sorted.
    let src = |elem: &str, hi: &str| {
        format!(
            "let n: {hi} = read()\nlet a: [0..n]0..=1_000_000_000 = read()\n\
             let s = sorted([{elem} for i in 0..n])\nprint(s[0].1, s[n - 1].1)\n"
        )
    };
    let narrow = cpp(&src("(a[i], i)", "1..=2_000_000"));
    assert!(
        narrow.contains("std::vector<std::tuple<uint32_t, uint32_t>> s;"),
        "{narrow}"
    );
    assert!(narrow.contains(".reserve("), "{narrow}");
    assert!(narrow.contains("cpl::sorted<true>("), "{narrow}");
    // An unbounded loop variable or a computed field keeps its full width.
    let wide_i = cpp(&src("(a[i], i)", "Int"));
    assert!(wide_i.contains("std::tuple<uint32_t, ll>> s;"), "{wide_i}");
    let wide_sum = cpp(&src("(a[i] + 1, i)", "1..=2_000_000"));
    assert!(
        wide_sum.contains("std::tuple<ll, uint32_t>> s;"),
        "{wide_sum}"
    );
}

#[test]
fn a_bijection_inverse_is_written_straight_into_a_narrow_result() {
    let out = cpp("let n: 1..=200_000 = read()
domain P = 1..=n
let a: [0..n]P = read()
let at = a.inverse()
print(at[P(1)])
");
    assert!(
        out.contains("std::vector<int32_t> t") && out.contains("int32_t(-1)"),
        "the result takes 4 B a cell and -1 marks an unseen image:\n{out}"
    );
    assert!(!out.contains("OptInt"), "no table of optionals:\n{out}");
    assert!(out.contains("(missing image)"), "{out}");
    assert!(out.contains("(duplicate image)"), "{out}");
}

#[test]
fn a_rolling_hash_keeps_its_prefixes_and_powers_as_two_pair_lists() {
    let out = cpp("use std.strings.{RollingHash}
let s: Str = read()
let h = RollingHash(s)
print(yes(h.slice(0, s.len) == h.all))
");
    assert!(
        out.contains("std::vector<std::tuple<ll, ll>> pre;")
            && out.contains("std::vector<std::tuple<ll, ll>> pow;"),
        "{out}"
    );
    assert!(!out.contains("pow1"), "{out}");
}

#[test]
fn a_joined_list_sorted_by_a_key_keeps_its_narrow_cells_and_is_moved() {
    let src = |cost: &str| {
        format!(
            "let n: 1..=1_000_000 = read()\ndomain F = 0..n\ndomain G = 0..n\n\
             let f: [F]{cost} = read()\nlet g: [G]{cost} = read()\n\
             enum Link:\n    Flight(f: F)\n    Portal(g: G)\n\
             let links = [(f[x], Flight(x)) for x in F] + [(g[y], Portal(y)) for y in G]\n\
             var tot = 0\n\
             for (i, (c, l)) in links.sorted(by: l => l.0).items:\n    match l:\n        \
             Flight(x) => tot += i * x.ord\n        Portal(y) => tot -= i * y.ord\n\
             print(tot)\n"
        )
    };
    let narrow = cpp(&src("1..=100_000_000"));
    assert!(
        narrow.contains("std::vector<std::tuple<uint32_t, int>> links;"),
        "{narrow}"
    );
    assert!(narrow.contains("= std::move(links);"), "{narrow}");
    assert!(
        narrow.contains("[&](const std::tuple<ll, int>& l) -> ll"),
        "{narrow}"
    );
    let wide = cpp(&src("Int"));
    assert!(
        wide.contains("std::vector<std::tuple<ll, int>> links;"),
        "{wide}"
    );
}

#[test]
fn a_narrow_cell_holds_its_width_in_bytes() {
    use cpl_ir::IntWidth;
    for (w, bytes) in [
        (IntWidth::U8, 1),
        (IntWidth::I8, 1),
        (IntWidth::U16, 2),
        (IntWidth::I16, 2),
        (IntWidth::U32, 4),
        (IntWidth::I32, 4),
    ] {
        assert_eq!(IntWidth::cpp_bytes(w.cpp()), Some(bytes), "{}", w.cpp());
    }
    for (cell, bytes) in [
        ("ll", Some(8)),
        ("ull", Some(8)),
        ("double", Some(8)),
        ("int", Some(4)),
        ("char", Some(1)),
        ("bool", None),
    ] {
        assert_eq!(IntWidth::cpp_bytes(cell), bytes, "{cell}");
    }
}

#[test]
fn a_range_bounded_by_products_of_capped_lets_narrows_as_explain_sizes_it() {
    let out = cpp("let n0: 1..=15_000 = read()
let n = n0 + 1
let m = n * (n + 1) / 2
var t: [0..m]0..=2 * n = 0
var h: [0..n]-(n / 2)..n - 1 = 0
t[m - 1] = 2 * n
h[0] = -(n / 2)
print(t[m - 1], h[0])
");
    assert!(
        out.contains("std::vector<uint16_t> t;"),
        "`0..=2 * n` with `n <= 15001` fits 16 bits:\n{out}"
    );
    assert!(
        out.contains("std::vector<int16_t> h;"),
        "`-(n / 2)..n - 1` fits 16 signed bits:\n{out}"
    );
}

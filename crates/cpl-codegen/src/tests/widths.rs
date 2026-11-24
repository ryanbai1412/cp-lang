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
    assert!(
        widened.contains("std::vector<uint32_t> a")
            && widened.contains("(const std::vector<uint32_t>& xs)"),
        "expected array passed to [V]Int param to keep its width in the instance:\n{widened}"
    );
}

#[test]
fn ranged_2d_nested_and_tuple_elements_narrow_and_widen_conservatively() {
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
        grid.contains("ll(g("),
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
    assert!(
        widened.contains("cpl::Grid<uint16_t, 2> g")
            && widened.contains("(const cpl::Grid<uint16_t, 2>& g_"),
        "expected 2D grid passed to [R, C]Int to keep its width in the instance:\n{widened}"
    );
    assert!(
        widened.contains("std::vector<std::vector<ll>> mat"),
        "expected nested array whose row flows to [E]Int to widen:\n{widened}"
    );
    assert!(
        widened.contains("std::vector<std::tuple<uint16_t, int8_t>> pts")
            && widened.contains("(const std::vector<std::tuple<uint16_t, int8_t>>& pts_"),
        "expected tuple array passed to [V](Int, Int) to keep its width in the instance:\n{widened}"
    );
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

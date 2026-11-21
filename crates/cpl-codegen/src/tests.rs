//! The shape of the C++: what the differential tests can't see, because
//! slower or uglier C++ prints the same answers.

use crate::emit;

fn cpp(source: &str) -> String {
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    emit(&program, "t.cp", Some(source))
}

/// The statements of the program: the C++ after the runtime it leans on.
fn body(source: &str) -> String {
    let cpp = cpp(source);
    let start = cpp.find("void solve()").expect("a solve fn");
    cpp[start..].to_string()
}

#[test]
fn division_by_a_positive_number_of_a_non_negative_one_is_plain_cpp() {
    let body = body(
        "let a: 0..=100 = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b, a % 1000000007)\n",
    );
    assert!(body.contains("cpl::put_int(a / b)"), "{body}");
    assert!(body.contains("cpl::put_int(a % b)"), "{body}");
    assert!(body.contains("cpl::put_int(a % 1000000007)"), "{body}");
    assert!(
        !body.contains("cpl::ediv") && !body.contains("cpl::divp"),
        "{body}"
    );
}

#[test]
fn a_dividend_that_may_be_negative_gets_the_branchless_fix() {
    let source = "let a: Int = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b)\n";
    let body = body(source);
    assert!(
        body.contains("cpl::divp(a, b)") && body.contains("cpl::modp(a, b)"),
        "{body}"
    );
    let cpp = cpp(source);
    assert!(
        cpp.contains("inline ll modp(ll a, ll b)") && cpp.contains("return r + ((r >> 63) & b);"),
        "{cpp}"
    );
    // The helpers it does not call are not in the file.
    assert!(!cpp.contains("inline ll emod"), "{cpp}");
}

#[test]
fn a_divisor_of_unknown_sign_gets_the_full_euclidean_helpers() {
    let body = body("let a: Int = read()\nlet b: Int = read()\nprint(a / b, a % b)\n");
    assert!(
        body.contains("cpl::ediv(a, b)") && body.contains("cpl::emod(a, b)"),
        "{body}"
    );
}

#[test]
fn a_variable_assigned_later_is_not_trusted_to_keep_its_range() {
    let body = body("var a: 0..=100 = read()\na -= 500\nprint(a % 7)\n");
    assert!(body.contains("cpl::modp(a, 7)"), "{body}");
}

#[test]
fn loop_counters_and_lengths_are_known_non_negative() {
    let body =
        body("let xs: List<Int> = [1, 2, 3]\nfor i in 0..10:\n    print(i % 3, xs.len / 2)\n");
    assert!(body.contains("cpl::put_int(i % 3)"), "{body}");
    assert!(body.contains("(ll)xs.size() / 2"), "{body}");
}

#[test]
fn the_file_opens_with_its_source_as_comments() {
    let source = "// ends in a backslash \\\nprint(1)\n";
    let cpp = cpp(source);
    assert!(cpp.starts_with("// t.cp: C++17 written by cpl "), "{cpp}");
    assert!(cpp.contains("//     print(1)\n"), "{cpp}");
    // A `\` at the end of a `//` line would make the next line a comment too.
    assert!(cpp.lines().all(|line| !line.ends_with('\\')), "{cpp}");
}

#[test]
fn only_the_runtime_a_program_uses_is_written() {
    let hello = cpp("print(\"hello\")\n");
    assert!(
        !hello.contains("read_int")
            && !hello.contains("struct Grid")
            && !hello.contains("put_float"),
        "{hello}"
    );
    // About 1 kB of it runs the program on a thread with the judge's stack.
    assert!(hello.len() < 2500, "{} bytes for hello world", hello.len());
}

#[test]
fn strings_are_bytes_with_no_encoding_machinery() {
    let cpp = cpp("let s: Str = read()\nvar t: Str = \"\"\nfor c in s:\n    t.push(c)\nprint(t, s[0], s.len)\n");
    assert!(cpp.contains("using u8 = unsigned char;"), "{cpp}");
    for word in [
        "wchar", "char32_t", "char16_t", "codecvt", "locale", "utf", "UTF",
    ] {
        assert!(
            !cpp[cpp.find("#include").expect("includes")..].contains(word),
            "`{word}` in {cpp}"
        );
    }
}

#[test]
fn bool_elements_are_bytes_not_vector_bool() {
    let cpp = cpp("var seen: [10]Bool = false\nseen[3] = true\nprint(seen[3])\n");
    assert!(
        cpp.contains("std::vector<char>") && !cpp.contains("std::vector<bool>"),
        "{cpp}"
    );
}

#[test]
fn the_same_source_gives_the_same_bytes() {
    let source = "var m: Map<Str, Int> = []\nm[\"b\"] = 1\nm[\"a\"] = 2\nvar s: Set<Int> = []\ns.insert(3)\nprint(m, s)\n";
    assert_eq!(cpp(source), cpp(source));
}

#[test]
fn without_its_source_the_file_says_so_and_is_otherwise_the_same() {
    let source = "let n: Int = read()\nprint(n + 1)\n";
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let program =
        cpl_ir::lower(&db.analyze("t.cp").expect("the file was just set")).expect("lowers");
    let (with, without) = (
        emit(&program, "t.cp", Some(source)),
        emit(&program, "t.cp", None),
    );
    assert!(
        without.starts_with("// t.cp: C++17 written by cpl "),
        "{without}"
    );
    assert!(
        without.contains("left out") && !without.contains("print(n + 1)"),
        "{without}"
    );
    let code = |cpp: &str| cpp[cpp.find("#include").expect("includes")..].to_string();
    assert_eq!(code(&with), code(&without));
}

#[test]
fn hash_map_uses_salted_splitmix64_cphash() {
    let cpp = cpp("var hm: HashMap<Int, Int> = []\nhm[42] = 7\nprint(hm.get(42) ?? 0)\n");
    assert!(cpp.contains("struct CpHash"), "{cpp}");
    assert!(cpp.contains("splitmix64"), "{cpp}");
    assert!(cpp.contains("steady_clock"), "{cpp}");
    assert!(cpp.len() < 65536, "{} bytes", cpp.len());
}

#[test]
fn generics_and_lends_emit_compact_readable_cpp() {
    let source = "\
struct Box<T>:
    x: T

impl<T> Box<T>:
    fn get(self) -> T: self.x
    fn mut_ref(mut self) lends mut T:
        lend mut self.x

var b = Box(x: 10)
b.mut_ref() += 5
print(b.get())
";
    let cpp = cpp(source);
    assert!(cpp.starts_with("// t.cp: C++17 written by cpl "), "{cpp}");
    assert!(cpp.contains("//     struct Box<T>:"), "{cpp}");
    assert!(cpp.len() < 65536, "{} bytes", cpp.len());
}

/// The C++ type `name` is declared with.
fn decl(cpp: &str, name: &str) -> String {
    let needle = format!(" {name}");
    let program = cpp.find("namespace prog").expect("the program");
    let line = cpp[program..]
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .find(|l| {
            l.split(['(', ';', '='])
                .next()
                .is_some_and(|head| head.trim_end().ends_with(&needle))
        })
        .unwrap_or_else(|| panic!("no declaration of `{name}`:\n{cpp}"));
    line.trim()
        .split(&needle)
        .next()
        .unwrap_or_default()
        .to_string()
}

const TABLE: &str = "let n: 1..=100 = read()\ndomain V = 0..n\nlet w: [V]0..=1000 = read()\nlet any: Int = read()\nvar dp: [V]?Int = none\n";

fn table(rest: &str) -> String {
    let cpp = cpp(&format!("{TABLE}{rest}\nprint(dp[V.first] ?? -1)\n"));
    decl(&cpp, "dp")
}

#[test]
fn an_optional_of_a_position_a_bool_or_a_char_is_one_scalar() {
    let cpp = cpp(
        "let n: 1..=9 = read()\ndomain V = 0..n\nvar p: [V]?V = none\nvar b: [V]?Bool = none\nvar c: [V]?Char = none\n\
         p[V.first] = V.first + 1\nb[V.first] = true\nc[V.first] = 'a'\n\
         print(p[V.first] ?? V.last, b[V.first] ?? false, c[V.first] ?? '-')\n",
    );
    assert_eq!(decl(&cpp, "p"), "std::vector<cpl::OptPos>", "{cpp}");
    assert_eq!(decl(&cpp, "b"), "std::vector<cpl::OptBool>", "{cpp}");
    assert_eq!(decl(&cpp, "c"), "std::vector<cpl::OptChar>", "{cpp}");
    assert!(cpp.contains("using OptPos = Niche<int, int, -1>;"), "{cpp}");
    assert!(cpp.contains("using OptBool = Niche<bool, u8, 2>;"), "{cpp}");
    assert!(
        cpp.contains("using OptChar = Niche<u8, short, -1>;"),
        "{cpp}"
    );
    // No Int table was packed, so its alias is not in the file.
    assert!(!cpp.contains("OptInt"), "{cpp}");
}

#[test]
fn a_program_without_optionals_carries_none_of_the_packing() {
    let cpp = cpp("let n: 1..=9 = read()\nprint(n)\n");
    assert!(!cpp.contains("Niche"), "{cpp}");
}

#[test]
fn the_inside_of_a_small_optional_taken_as_a_place_keeps_it_unpacked() {
    let cpp = cpp("var f: ?Bool = none\nvar c: ?Char = none\n(f ??= false) = true\nprint(f ?? false, c ?? '-')\n");
    assert_eq!(decl(&cpp, "f"), "std::optional<bool>", "{cpp}");
    // A Char is still packed: only the Bool is opened in place.
    assert_eq!(decl(&cpp, "c"), "cpl::OptChar", "{cpp}");
}

#[test]
fn every_optional_int_is_one_int_with_int64_min_as_none() {
    const PACKED: &str = "std::vector<cpl::OptInt>";
    // Whatever is stored: unbounded input, sums, other tables.
    assert_eq!(table("dp[V.first] = any"), PACKED);
    assert_eq!(table("dp[V.first] = Int.MIN"), PACKED);
    assert_eq!(table("dp[V.first] = -1\ndp[V.last] = (dp[V.first] ?? 0) * 2"), PACKED);
    assert_eq!(table("fn f() -> Int: any\ndp[V.first] = f()"), PACKED);
    // Used as a whole value: passed, mutated through, looped over, copied.
    assert_eq!(table("fn f(xs: [V]?Int) -> Int: xs[V.first] ?? 0\nprint(f(dp))"), PACKED);
    assert_eq!(table("fn f(mut xs: [V]?Int):\n    xs[V.first] = 1\nf(mut dp)"), PACKED);
    assert_eq!(table("for x in dp: print(x ?? 0)"), PACKED);
    assert_eq!(table("let copy = dp\nprint(copy[V.first] ?? 0)"), PACKED);
    // A list filled by `push`, a grid, a local, a parameter and a result.
    let cpp = cpp("fn half(x: ?Int) -> ?Int: x\nlet n: 0..=9 = read()\nvar xs: List<?Int> = []\nvar g: [n, n]?Int = none\nfor i in 0..n:\n    xs.push(none)\n    xs.push(i)\n    g[i, 0] = i\nlet best: ?Int = xs.pop()\nprint(xs.len, xs[0] ?? -1, g[0, 0] ?? -1, half(best) ?? -1)\n");
    assert_eq!(decl(&cpp, "xs"), "std::vector<cpl::OptInt>", "{cpp}");
    assert_eq!(decl(&cpp, "g"), "cpl::Grid<cpl::OptInt, 2>", "{cpp}");
    assert_eq!(decl(&cpp, "best"), "cpl::OptInt", "{cpp}");
    assert!(cpp.contains("cpl::OptInt half(const cpl::OptInt& x)"), "{cpp}");
    assert!(cpp.contains("using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;"), "{cpp}");
    assert!(!cpp.contains("std::optional<ll>"), "{cpp}");
}

#[test]
fn a_fn_of_the_judges_grader_is_declared_in_its_types() {
    let cpp = cpp(
        "@grader fn ask(i: Int) -> List<Int>\n@grader fn mark(ok: Bool, c: Char, label: Str)\n\
         @grader\nfn best(n: Int) -> (Int, List<Int>):\n    mark(true, 'a', \"x\")\n    (ask(n)[0], ask(0))\n",
    );
    // The judge's fns, as its grader writes them, ahead of the program.
    let runtime = cpp.find("namespace cpl").expect("a runtime");
    for decl in [
        "std::vector<int> ask(int i);",
        "void mark(bool ok, char c, std::string label);",
    ] {
        assert!(cpp[..runtime].contains(decl), "{cpp}");
    }
    assert!(
        cpp.contains("return cpl::Judge<std::vector<ll>>::from(::ask(cpl::Judge<ll>::to(i)));"),
        "{cpp}"
    );
    // The fn the grader calls, in its types, and no `main` beside the grader's.
    assert!(
        cpp.contains("\nstd::tuple<int, std::vector<int>> best(int n) {\n  cpl_start();"),
        "{cpp}"
    );
    assert!(!cpp.contains("int main()"), "{cpp}");
}

#[test]
fn a_program_without_grader_fns_has_a_main_and_no_judge_types() {
    let cpp = cpp("print(1)\n");
    assert!(
        cpp.contains("int main()") && !cpp.contains("Judge"),
        "{cpp}"
    );
}

#[test]
fn a_map_whose_order_never_shows_is_a_hash_table() {
    let blind = cpp("var m = Map<Int, Int>()\nvar s = Set<Int>()\nm[42] = 7\ns.insert(3)\nprint(m.get(42) ?? 0, 3 in s, m.len)\n");
    assert!(
        blind.contains("cpl::HashMap<ll, ll, cpl::CpHash>"),
        "{blind}"
    );
    assert!(
        blind.contains("cpl::HashSet<ll, cpl::CpHash>"),
        "{blind}"
    );
    assert!(!blind.contains("std::map<ll, ll>"), "{blind}");
    for ordered in [
        "for (k, v) in m: print(k, v)",
        "print(m)",
        "print(m.first_ge(3))",
        "print(m.first)",
        "print(m.pop_last())",
        "let ks = sorted(k for (k, _) in m)\nprint(ks)",
        "var other = Map<Int, Int>()\nprint(m == other)",
    ] {
        let seen = cpp(&format!("var m = Map<Int, Int>()\nm[42] = 7\n{ordered}\n"));
        assert!(seen.contains("std::map<ll, ll>"), "{ordered}: {seen}");
        assert!(
            !seen.contains("cpl::HashMap<ll, ll"),
            "{ordered}: {seen}"
        );
    }
}

#[test]
fn a_key_tested_and_then_read_is_looked_up_once() {
    let once =
        cpp("var m = Map<Int, Int>()\nm[4] = 7\nlet x: Int = read()\nif x in m: print(m[x] + 1)\n");
    assert!(
        once.contains("= m.find(x);") && once.contains("->second"),
        "{once}"
    );
    assert!(!once.contains("m.at(x)"), "{once}");
    // A body that changes the map keeps the two lookups.
    let twice = cpp("var m = Map<Int, Int>()\nm[4] = 7\nlet x: Int = read()\nif x in m:\n    m.remove(x + 1)\n    print(m[x])\n");
    assert!(twice.contains("m.at(x)"), "{twice}");
}

#[test]
fn a_list_index_inside_unchecked_has_no_bounds_test() {
    let checked = cpp("var xs = [1, 2, 3]\nlet i: Int = read()\nxs[i] = xs[i] + 1\nprint(xs)\n");
    assert!(checked.contains("cpl::at(xs, i)"), "{checked}");
    let raw = cpp(
        "var xs = [1, 2, 3]\nlet i: Int = read()\nunchecked:\n    xs[i] = xs[i] + 1\nprint(xs)\n",
    );
    assert!(raw.contains("xs[i] = xs[i] + 1"), "{raw}");
    assert!(!raw.contains("cpl::at(xs, i)"), "{raw}");
}

#[test]
fn a_local_list_that_stays_in_its_fn_is_one_buffer() {
    let fixture = include_str!("../../cpl-cli/tests/programs/reused_local_lists.cp");
    let cpp = cpp(fixture);
    assert!(
        cpp.contains("static std::vector<ll> xs;\n  xs.clear();"),
        "{cpp}"
    );
    assert!(!cpp.contains("static std::vector<ll> seen;"), "{cpp}");
    assert!(!cpp.contains("static std::vector<ll> out;"), "{cpp}");
}

#[test]
fn a_const_as_a_type_argument_is_its_number() {
    let cpp = cpp("const MOD = 998244353\ntype Z = Mod<998244353>\nlet c = Comb<Mod<MOD>>()\nlet x: Z = Z(5)\nprint(c.choose(5, 2), x * x)\n");
    assert_eq!(cpp.matches("\nstruct Mod_").count(), 1, "{cpp}");
}

#[test]
fn a_list_a_comprehension_built_moves_into_the_variable() {
    let cpp = cpp("let n: Int = read()
var zs: List<Int> = []
for r in 0..3:
    zs = [i * r for i in 0..n]
    let ys = [z + 1 for z in zs]
    zs = ys
print(zs)
");
    assert!(cpp.contains("zs = std::move(t0);"), "{cpp}");
    assert!(cpp.contains("zs = ys;"), "a named list is copied: {cpp}");
}

#[test]
fn an_index_of_the_arrays_own_domain_has_no_bounds_test() {
    // A `D` is one of `D`'s positions by construction; an `Int` is not.
    let src = "let n: Int = read()\ndomain D = 0..n\ndomain R = 0..n\nvar a: [D]Int = 0\nvar g: [R, D]Int = 0\n\
               var b: [0..n]Int = 0\nvar h: [0..n, 0..n]Int = 0\nlet k: Int = read()\n\
               for r in R:\n    for d in D:\n        a[d] += g[r, d]\n        g[r, d] = a[d]\n\
               b[k] = h[k, k]\nprint(a, g, b)\n";
    let out = cpp(src);
    assert!(out.contains("a[d] += g(r, d);"), "{out}");
    assert!(out.contains("g(r, d) = a[d];"), "{out}");
    assert!(out.contains("cpl::at(b, k) = h.ck(k, k);"), "{out}");
}

#[test]
fn a_let_of_a_list_nothing_changes_is_a_reference_not_a_copy() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "var rows: List<List<Int>> = [[i * j for j in 0..=i] for i in 0..n]\n",
        "fn last_of(r: Int) -> Int:\n",
        "    let row = rows[r]\n",
        "    row[row.len - 1]\n",
        "fn grow(r: Int) -> Int:\n",
        "    let kept = rows[r]\n",
        "    rows[r].push(100)\n",
        "    kept.len\n",
        "print(last_of(2), grow(1))\n",
        "let vals = compress([30, 10, 20, 10])\n",
        "print(vals.at(20))\n",
    ));
    assert!(
        cpp.contains("const std::vector<ll>& row = cpl::at(rows, r);"),
        "{cpp}"
    );
    // `grow` changes `rows`: `kept` is what the row was.
    assert!(
        cpp.contains("std::vector<ll> kept = cpl::at(rows, r);"),
        "{cpp}"
    );
    assert!(!cpp.contains("const std::vector<ll>& kept"), "{cpp}");
    // `vals.at(x)` searches the field where it is.
    assert!(!cpp.contains("= self.vals;"), "{cpp}");
}

#[test]
fn a_let_at_the_top_level_is_a_reference_while_nothing_after_it_changes_the_list() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "var rows: [0..n]List<Int> = []\n",
        "fn bump(i: Int):\n    rows[i].push(100)\n",
        "fn size(i: Int) -> Int: rows[i].len\n",
        "var total = 0\n",
        "for i in 0..n:\n    let row = rows[i]\n    total += row.len + size(i)\n",
        "for i in 0..n:\n    let before = rows[i]\n    bump(i)\n    total += before.len\n",
        "for i in 0..n:\n    let kept = rows[i]\n    rows[i].push(7)\n    total += kept.len\n",
        "for i in 0..n:\n    let was = rows[i]\n    let f = k => rows[k].push(3)\n    f(i)\n    total += was.len\n",
        "fn inside(i: Int) -> Int:\n",
        "    if i > 0:\n        let a = rows[i]\n        return a.len + size(i)\n",
        "    let b = rows[i]\n    bump(i)\n    b.len\n",
        "print(total, inside(0))\n",
    ));
    // `size` reads `rows` and changes nothing.
    assert!(
        cpp.contains("const std::vector<ll>& row = cpl::at(rows, "),
        "{cpp}"
    );
    assert!(
        cpp.contains("const std::vector<ll>& a = cpl::at(rows, i);"),
        "{cpp}"
    );
    // A fn that changes `rows`, a store, and a fn value that may do
    // either: each name is what the row was.
    for copied in ["before", "kept", "was", "b"] {
        assert!(
            cpp.contains(&format!("std::vector<ll> {copied} = cpl::at(rows, ")),
            "{copied}: {cpp}"
        );
        assert!(!cpp.contains(&format!("& {copied} = ")), "{copied}: {cpp}");
    }
}

#[test]
fn an_optional_struct_with_an_int_field_has_no_flag() {
    let cpp = cpp(concat!(
        "struct Edge(to: Int, w: Int)\n",
        "struct Tag(c: Char)\n",
        "var best: ?Edge = none\n",
        "best = Edge(3, 4)\n",
        "var t: ?Tag = none\n",
        "t = Tag('a')\n",
        "print(best == none, t == none)\n",
    ));
    assert!(
        cpp.contains("cpl::OptField<&Edge::to> best"),
        "{cpp}"
    );
    // A `Char` has no free value, so `?Tag` keeps its flag.
    assert!(cpp.contains("std::optional<Tag> t"), "{cpp}");
}

#[test]
fn an_optional_tuple_with_a_position_has_no_flag() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "var far: ?(depth: Int, child: V) = none\n",
        "far = (3, V.first)\n",
        "var plain: ?(Int, Int) = none\n",
        "plain = (1, 2)\n",
        "print(far, plain)\n",
    ));
    assert!(
        cpp.contains("cpl::OptTuple<std::tuple<ll, int>, 1> far"),
        "{cpp}"
    );
    assert!(
        cpp.contains("template <class T, size_t K> struct OptTuple {"),
        "{cpp}"
    );
    // No `Int` of a tuple is free to say `none`.
    assert!(
        cpp.contains("std::optional<std::tuple<ll, ll>> plain"),
        "{cpp}"
    );
}

/// `for j in 0..b.len:` in a fn indexes `b[j]` with no check, as `L0414`
/// promises, as long as the body leaves `b` at that length.
#[test]
fn a_loop_to_the_length_of_a_list_indexes_it_unchecked() {
    let dot = |body: &str| {
        cpp(&format!(
            "fn grow(mut xs: List<Int>): xs.push(0)\n\
             fn dot(a: List<Int>, mut b: List<Int>) -> Int:\n    var s = 0\n    for j in 0..b.len:\n{body}    s\n\
             var ys = [3, 4]\nprint(dot([1, 2], mut ys))\n"
        ))
    };
    let kept = dot("        s += a[0] * b[j]\n        b[j] += 1\n");
    assert!(kept.contains("s += cpl::at(a, 0) * b[j];"), "{kept}");
    assert!(kept.contains("b[j] += 1;"), "{kept}");
    for body in [
        "        s += b[j]\n        if s > 9: b.pop()\n",
        "        s += b[j]\n        grow(mut b)\n",
        "        s += b[j]\n        b = [1]\n",
        "        s += b[j + 1]\n",
    ] {
        let out = dot(body);
        assert!(out.contains("cpl::at(b, j"), "{body}: {out}");
    }
}

#[test]
fn a_fn_is_declared_ahead_only_when_it_is_named_before_it_is_defined() {
    let cpp = cpp("fn first(n: Int) -> Int: n + 1\n\
         fn even(n: Int) -> Bool: n == 0 or odd(n - 1)\n\
         fn odd(n: Int) -> Bool: n != 0 and even(n - 1)\n\
         print(first(2), even(4))\n");
    assert!(cpp.contains("\nbool odd(ll n);\n"), "{cpp}");
    assert!(
        !cpp.contains("\nbool even(ll n);\n") && !cpp.contains("\nll first(ll n);\n"),
        "{cpp}"
    );
}

#[test]
fn interactive_flushes_after_each_print() {
    let said = "cpl::put_line(); cpl::flush(); fflush(stdout);";
    assert!(cpp("@!interactive\nprint(1)\n").contains(said));
    assert!(!cpp("print(1)\n").contains(said));
}

#[test]
fn a_ranged_read_checks_its_bounds() {
    let ranged = cpp("let k: 1..=5 = read()\nprint(k)\n");
    assert!(ranged.contains("cpl::read_in(1, 5)"), "{ranged}");
    let plain = cpp("let k: Int = read()\nprint(k)\n");
    assert!(!plain.contains("read_in("), "{plain}");
}

#[test]
fn the_program_runs_on_a_thread_with_the_profile_stack() {
    let source = "let n: Int = read()\nprint(n)\n";
    let cpp = cpp(source);
    assert!(cpp.contains("a 1024 MB stack."), "{cpp}");
    assert!(cpp.contains("static void cpl_main() {"), "{cpp}");
    assert!(cpp.contains("for (size_t n = (size_t)1024 << 20;"), "{cpp}");
    assert!(
        cpp.contains("pthread_attr_setstacksize") && !cpp.contains("CreateThread"),
        "{cpp}"
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let windows = crate::Target {
        windows: true,
        ..crate::Target::default()
    };
    let cpp = crate::emit_for(&program, "t.cp", None, &windows);
    assert!(
        cpp.contains("#ifdef _WIN32") && cpp.contains("CreateThread(0, n,"),
        "{cpp}"
    );
    assert!(cpp.contains("#else\n#include <pthread.h>"), "{cpp}");
    // No call depth is counted, in any build.
    assert!(!cpp.contains("depth"), "{cpp}");
}

#[test]
fn a_judge_without_threads_has_main_run_the_program() {
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", "print(1)\n");
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let target = crate::Target {
        thread_stack: None,
        ..crate::Target::default()
    };
    let cpp = crate::emit_for(&program, "t.cp", None, &target);
    assert!(cpp.contains("The judge allows no threads"), "{cpp}");
    assert!(
        cpp.contains("int main() {") && !cpp.contains("pthread") && !cpp.contains("cpl_main"),
        "{cpp}"
    );
}

#[test]
fn unused_structs_and_their_trait_hooks_are_omitted() {
    let cpp = cpp(concat!(
        "struct Used(x: Int)\n",
        "struct Unused(x: Int)\n",
        "impl Ord for Unused:\n    fn lt(a, b) = a.x < b.x\n",
        "impl Show for Unused:\n    fn show(a) = \"{a.x}\"\n",
        "let u = Used(7)\n",
        "print(u.x)\n",
    ));
    assert!(cpp.contains("struct Used {"), "{cpp}");
    assert!(!cpp.contains("struct Unused {"), "{cpp}");
    assert!(!cpp.contains("Unused_lt"), "{cpp}");
    assert!(!cpp.contains("Unused_show"), "{cpp}");
}

#[test]
fn static_struct_and_param_ops_eliminate_std_function() {
    let src = concat!(
        "struct Box(op: (Int, Int) -> Int, base: Int)\n",
        "fn apply_box(b: Box, x: Int) -> Int = b.op(b.base, x)\n",
        "let b = Box((x, y) => x + y, 10)\n",
        "print(apply_box(b, 5))\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(!out.contains("std::function"), "{out}");
    assert!(out.contains("cpl_fn0("), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained.iter().any(|e| matches!(e.mode, crate::ClosureMode::StaticDirect { .. })),
        "{explained:?}"
    );
    assert!(
        explained.iter().all(|e| !matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
        "{explained:?}"
    );
}

#[test]
fn multi_closure_fn_param_lowers_to_template_functor() {
    let src = concat!(
        "fn twice(f: Int -> Int, x: Int) -> Int = f(f(x))\n",
        "let k = 3\n",
        "print(twice(x => x + 1, 2), twice(x => x + k, 2))\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(out.contains("template <class F0>"), "{out}");
    assert!(!out.contains("std::function"), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained.iter().any(|e| e.mode == crate::ClosureMode::StaticTemplate),
        "{explained:?}"
    );
}

#[test]
fn local_closure_lowers_to_auto_and_dynamic_field_falls_back_to_std_function() {
    let src = concat!(
        "struct Box(op: Int -> Int)\n",
        "fn run(k: Int) -> Int:\n",
        "    let loc = (x: Int) => x + k\n",
        "    loc(1)\n",
        "let k = 5\n",
        "let a = Box(x => x + 1)\n",
        "let b = Box(x => x + k)\n",
        "print(run(k), a.op(2), b.op(3))\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(out.contains("auto loc ="), "{out}");
    assert!(out.contains("std::function<ll(ll)> op;"), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained.iter().any(|e| e.mode == crate::ClosureMode::StaticLocalAuto),
        "{explained:?}"
    );
    assert!(
        explained
            .iter()
            .any(|e| matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
        "{explained:?}"
    );
}

#[test]
fn ranged_int_struct_fields_narrow_to_32_bits() {
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
    assert!(cpp.contains("uint32_t v = 0;"), "Mod<998244353> should narrow to uint32_t: {cpp}");
    assert!(cpp.contains("int32_t w = 0;"), "signed range should narrow to int32_t: {cpp}");
    assert!(cpp.contains("ll(a.v)"), "narrowed field reads should widen to ll: {cpp}");
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
fn cheap_pure_sort_key_avoids_sort_by_buffers() {
    let cpp = cpp(concat!(
        "struct Pt(x: Int, y: Int)\n",
        "var xs = [Pt(2, 1), Pt(1, 3)]\n",
        "xs.sort(by: p => (p.x, p.y))\n",
        "print(xs[0].x)\n",
    ));
    assert!(cpp.contains("std::stable_sort("), "{cpp}");
    assert!(!cpp.contains("cpl::sort_by("), "{cpp}");
}

#[test]
fn small_domain_bitset_uses_inline_row_and_fused_popcount() {
    let cpp = cpp(concat!(
        "let n: 1..=300 = read()\n",
        "domain V = 0..n\n",
        "let a = Bitset<V>.full\n",
        "let b = Bitset<V>.single(V(0))\n",
        "let c = Bitset<V>.full\n",
        "print((a & b & c).len)\n",
    ));
    assert!(cpp.contains("cpl::Row<unsigned long long, 5> w;"), "{cpp}");
    assert!(cpp.contains("__builtin_popcountll("), "{cpp}");
    assert!(!cpp.contains("_bitand(a, b)"), "{cpp}");
}
#[test]
fn higher_order_combinator_devirtualizes_to_zero_std_function() {
    let cpp = cpp(concat!(
        "use std.lazy.{Flip, flip_action}\n",
        "domain I = 0..4\n",
        "let m = Monoid<Int>(0, (a, b) => a + b)\n",
        "var st = LazySegTree<I, Flip<Int>>(flip_action<Int>(m), [I] i => Flip<Int>.of(i.ord % 2 == 0, 1, 0))\n",
        "st.apply(I(0)..I(3), true)\n",
        "print(st.all.now)\n",
    ));
    assert!(!cpp.contains("std::function"), "{cpp}");
    assert!(!cpp.contains("#include <functional>"), "{cpp}");
}

#[test]
fn struct_never_compared_emits_no_tie_operators() {
    let uncompared = cpp(concat!(
        "struct Pt(x: Int, y: Int)\n",
        "let p = Pt(3, 4)\n",
        "print(p.x + p.y)\n",
    ));
    assert!(
        !uncompared.contains("operator==("),
        "uncompared struct should not emit cpl_tie operators: {uncompared}"
    );
    let compared = cpp(concat!(
        "struct Pt(x: Int, y: Int)\n",
        "let a = Pt(1, 2)\n",
        "let b = Pt(3, 4)\n",
        "print(if a < b: 1 else: 0)\n",
    ));
    assert!(
        compared.contains("operator<("),
        "compared struct must emit cpl_tie operators: {compared}"
    );
}

#[test]
fn global_stored_once_with_a_literal_is_constexpr() {
    let out = cpp(concat!(
        "const M = 1_000_000_007\n",
        "let n = 500_000\n",
        "let big = true\n",
        "let x = read<Int>()\n",
        "let f = (y: Int) => y % M\n",
        "print(f(x) + n, big)\n",
    ));
    assert!(out.contains("static constexpr ll M = 1000000007;"), "{out}");
    assert!(out.contains("static constexpr ll n = 500000;"), "{out}");
    assert!(out.contains("static constexpr bool big = true;"), "{out}");
    assert!(!out.contains("  M = 1000000007;"), "constexpr M must not be stored again: {out}");
    let stored_twice = cpp(concat!("var m = 5\n", "m += read<Int>()\n", "print(m)\n"));
    assert!(!stored_twice.contains("constexpr ll m"), "{stored_twice}");
}

#[test]
fn a_str_grows_in_place_when_appended_to() {
    let body = body("let s: Str = read()\nvar t = \"\"\nfor c in s: t += c\nprint(t, s[1..], s[..=1])\n");
    assert!(body.contains("t.push_back((char)c)"), "{body}");
    assert!(!body.contains("Capture"), "{body}");
}

#[test]
fn a_yields_loop_whose_body_calls_a_fn_value_runs_as_the_fn_yields() {
    let src = |closure: &str| {
        format!(
            "var seen = 0\nfn upto(n: Int) -> yields Int:\n    for i in 0..n:\n        seen += 1\n        yield i * seen\n\
             fn total(n: Int, w: Int -> Int) -> Int:\n    var t = 0\n    for x in upto(n): t += w(x)\n    t\n\
             print(total(4, {closure}))\n"
        )
    };
    let pure = cpp(&src("x => x * 2"));
    assert!(pure.contains("cpl_each_upto(n, "), "{pure}");
    assert!(!pure.contains("= upto(n)"), "{pure}");
    // The body would change what `upto` reads between its yields.
    let bumps = cpp(&src("x => { seen += 10; x }"));
    assert!(bumps.contains("= upto(n)"), "{bumps}");
}

#[test]
fn narrowing_a_record_array_reads_the_source_in_place() {
    let body = body(concat!(
        "domain E = 0..3\n",
        "let edges: [E](u: Int, v: Int, w: Int) = read()\n",
        "fn ends(es: [E](u: Int, v: Int)) -> Int = es[E(0)].u + es[E(2)].v\n",
        "print(ends(edges))\n",
    ));
    assert!(!body.contains("= edges;"), "{body}");
    assert!(body.contains("& t"), "{body}");
}

#[test]
fn trie_row_field_niches_opt_int_and_avoids_temp_vector() {
    let out = cpp(concat!(
        "struct Trie(cnt: Int, kid: [0..2]?Int)\n",
        "var nodes = List<Trie>()\n",
        "fn add(bit: Int):\n",
        "    nodes.push(Trie(1, [0..2] _ => none))\n",
        "    let below = nodes.len - 1\n",
        "    nodes.push(Trie(1, [0..2] b => if b == bit: below else: none))\n",
        "add(read<Int>() & 1)\n",
        "print(nodes.len)\n",
    ));
    assert!(out.contains("cpl::Row<cpl::OptInt, 2> kid;"), "{out}");
    assert!(!out.contains("std::optional<ll>"), "{out}");
    assert!(
        !out.contains("-> std::vector<cpl::OptInt>"),
        "Row field comprehension must construct cpl::Row directly without a temporary std::vector: {out}"
    );
}

#[test]
fn a_param_returned_in_a_struct_is_taken_by_value_and_moved_in() {
    let cpp = cpp("struct Pack(xs: List<Int>, ys: List<Int>)
fn pack(xs: List<Int>, ys: List<Int>) -> Pack:
    Pack(xs, ys)
fn twice(xs: List<Int>) -> Pack = Pack(xs, xs)
fn make(n: Int) -> Pack:
    let a = [i for i in 0..n]
    let b = [i * i for i in 0..n]
    return pack(a, b)
let n: 1..=100 = read()
let p = make(n)
let q = twice(p.xs)
print(p.xs.len + p.ys.len, q.xs.len)
");
    assert!(cpp.contains("Pack pack(std::vector<ll> xs, std::vector<ll> ys) {"), "{cpp}");
    assert!(cpp.contains("return Pack{std::move(xs), std::move(ys)};"), "{cpp}");
    assert!(cpp.contains("return ::prog::pack(std::move(a), std::move(b));"), "{cpp}");
    assert!(cpp.contains("Pack twice(const std::vector<ll>& xs) {"), "named twice: copied, not sunk: {cpp}");
    assert!(cpp.contains("::prog::twice(p.xs)"), "a global is not moved from: {cpp}");
}

#[test]
fn closure_mutating_a_captured_local_is_not_hoisted_static() {
    let cpp = cpp(concat!(
        "fn each(f: (Int) -> ()):\n",
        "    for i in 0..3: f(i)\n",
        "fn run() -> Int:\n",
        "    var xs = List<Int>()\n",
        "    each(x => xs.push(x))\n",
        "    var ys = List<Int>()\n",
        "    each(x => push_to(mut ys, x))\n",
        "    xs.len + ys.len\n",
        "fn push_to(mut l: List<Int>, x: Int):\n",
        "    l.push(x)\n",
        "print(run())\n",
    ));
    assert!(!cpp.contains("static auto cpl_fn"), "{cpp}");
}

#[test]
fn an_array_built_by_a_comprehension_moves_into_its_variable() {
    let cpp = cpp("let n: 1..=10 = read()
domain V = 0..n
var d: [V]Int = [v.ord for v in V]
d[V.first] += 5
print(d.sum())
");
    assert!(cpp.contains("d = std::move(t"), "{cpp}");
}

//! The shape of the C++: what the differential tests can't see, because
//! slower or uglier C++ prints the same answers.

use crate::emit;

mod arith;
mod closures;
mod copies;
mod devirt;
mod lend;
mod loops;
mod optionals;
mod runtime;
mod uses;
mod widths;

pub(crate) fn cpp(source: &str) -> String {
    let mut db = cpl_check::Session::new();
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

pub(crate) fn lower(source: &str) -> cpl_ir::Program {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    cpl_ir::lower(&db.analyze("t.cp").expect("the file was just set")).expect("lowers")
}

/// The statements of the program: the C++ after the runtime it leans on.
fn body(source: &str) -> String {
    let cpp = cpp(source);
    let start = cpp.find("void solve()").expect("a solve fn");
    cpp[start..].to_string()
}

#[test]
fn the_file_opens_with_its_source_as_comments() {
    let source = "// ends in a backslash \\\nprint(1)\n";
    let cpp = cpp(source);
    assert!(cpp.starts_with("// t.cp: C++17 written by cpl "), "{cpp}");
    assert!(
        cpp.lines()
            .next()
            .unwrap()
            .contains("(https://cp-lang.org)"),
        "{cpp}"
    );
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
fn const_length_fixed_arrays_and_loop_literals_use_std_array() {
    let source = "\
enum Side:
    Left
    Mid
    Right
fn sum3(xs: [3]Int) -> Int = xs[0] + xs[1] + xs[2]
let n: 1..=100 = read()
var dp: [0..3]Int = [0, 0, 0]
var cnt: [Side]Int = 0
for i in 0..n:
    var nxt: [0..3]Int = [dp[1] + 1, dp[2] + 2, dp[0] + 3]
    dp = nxt
    cnt[Side.Left] += 1
var s = sum3([dp[0], dp[1], dp[2]])
for x in [dp[0], dp[1], dp[2]]:
    s += x
print(s, cnt[Side.Left])
";
    let out = cpp(source);
    let body = &out[out.find("namespace prog {").expect("prog")..];
    assert!(
        body.contains("std::array<ll, 3>") && !body.contains("std::vector<"),
        "{body}"
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
    let mut db = cpl_check::Session::new();
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
fn a_float_key_hashes_as_std_hash_does_with_zero_and_minus_zero_alike() {
    let cpp = cpp("var hm: HashMap<F64, Int> = []\nhm[1.5] = 7\nprint(hm.get(-0.0) ?? 0)\n");
    assert!(
        cpp.contains("operator()(double x) const { return (*this)((ll)std::hash<double>()(x)); }"),
        "{cpp}"
    );
}

#[test]
fn generics_and_lends_emit_compact_readable_cpp() {
    let source = "\
struct Box<T>:
    x: T

impl<T> Box<T>:
    fn get(self) -> T: self.x
    fn mut_ref(mut self) -> lends mut T:
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

const TABLE: &str = "let n: 1..=100 = read()\ndomain V = 0..n\nlet w: [V]0..=1000 = read()\nlet any: Int = read()\nvar dp: [V]?Int = None\n";

fn table(rest: &str) -> String {
    let cpp = cpp(&format!("{TABLE}{rest}\nprint(dp[V.first!] ?? -1)\n"));
    decl(&cpp, "dp")
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
    let blind = cpp("var m = Map<Int, Int>()\nvar s = Set<Int>()\nm[42] = 7\ns.insert(3)\nprint(m.get(42) ?? 0, yes(3 in s), m.len)\n");
    assert!(
        blind.contains("cpl::HashMap<ll, ll, cpl::CpHash>"),
        "{blind}"
    );
    assert!(blind.contains("cpl::HashSet<ll, cpl::CpHash>"), "{blind}");
    assert!(!blind.contains("std::map<ll, ll>"), "{blind}");
    for ordered in [
        "for (k, v) in m: print(k, v)",
        "print(m)",
        "print(m.first_ge(3))",
        "print(m.first)",
        "print(m.pop_last())",
        "let ks = sorted(k for (k, _) in m)\nprint(ks)",
        "var other = Map<Int, Int>()\nprint(yes(m == other))",
    ] {
        let seen = cpp(&format!("var m = Map<Int, Int>()\nm[42] = 7\n{ordered}\n"));
        assert!(seen.contains("std::map<ll, ll>"), "{ordered}: {seen}");
        assert!(!seen.contains("cpl::HashMap<ll, ll"), "{ordered}: {seen}");
    }
}

#[test]
fn a_memo_over_bounded_ranges_is_a_flat_table_alone() {
    let bounded = cpp("let a: 0..=8000 = read()\nlet b: 0..=399 = read()\n@memo\nfn f(k: 0..=a, s: 0..=b) -> Int:\n    if k == 0 or s == 0: return 1\n    f(k - 1, s) + f(k, s - 1)\nprint(f(a, b))\n");
    assert!(
        !bounded.contains("HashMap") && !bounded.contains("CpHash"),
        "{bounded}"
    );
    let open = cpp("let a: Int = read()\n@memo\nfn f(k: 0..=a) -> Int:\n    if k == 0: return 1\n    f(k - 1)\nprint(f(a))\n");
    assert!(open.contains("cpl::HashMap"), "{open}");
}

#[test]
fn a_memo_over_ranges_inferred_from_reads_is_a_flat_table_alone() {
    let dream = cpp(include_str!("../../../corpus/problems/cf2115c/dream.cp"));
    assert!(!dream.contains("CpHash"), "{dream}");
}

#[test]
fn a_product_domain_loop_is_nested_int_loops() {
    let head = "let h: 1..=50 = read()\nlet w: 1..=50 = read()\n\
                domain Row = 0..h\ndomain Col = 0..w\ndomain Square = (Row, Col)\n\
                let g: [Row, Col]Char = read()\nvar n = 0\n";
    let product = cpp(&format!(
        "{head}for (r, c) in Square if g[r, c] == '#':\n    n += 1\nprint(n)\n"
    ));
    let nested = cpp(&format!(
        "{head}for r in Row:\n    for c in Col if g[r, c] == '#':\n        n += 1\nprint(n)\n"
    ));
    assert!(
        !product.contains("ediv") && !product.contains("emod"),
        "{product}"
    );
    assert!(
        product.len() <= nested.len() + 200,
        "{} vs {}\n{product}",
        product.len(),
        nested.len()
    );
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
    let raw_at = cpp(
        "let xs = [1, 2, 3]\nlet i: Int = read()\nvar v = 0\nunchecked:\n    v = xs.at(i)\nprint(v)\n",
    );
    assert!(raw_at.contains("v = xs[i];"), "{raw_at}");
    assert!(!raw_at.contains("cpl::at(xs, i)"), "{raw_at}");
}

#[test]
fn domain_nth_inside_unchecked_omits_cpl_pos() {
    let out = cpp(r#"
let n = 10
domain I = 0..n
let a = [I] _ => 1
var s = 0
unchecked:
    for i in 0..n:
        s += a[I.nth(i)]
print(s)
"#);
    assert!(
        !out.contains("cpl::pos("),
        "`D.nth(i)` inside `unchecked:` should not emit `cpl::pos`:\n{out}"
    );
}

#[test]
fn unchecked_drops_domain_positions_but_keeps_range_stores() {
    let src = |block: &str| {
        format!(
            "domain D = 1..=10\nlet i: Int = read()\nvar a: [D]Int = 0\nvar x: 0..=100 = 0\n{block}print(x)\n"
        )
    };
    let body = "a[D.nth(i)] = 1\na[D(i)] = 2\nx = i\n";
    let checked = cpp(&src(body));
    assert!(checked.contains("cpl::pos(i, D_len)"), "{checked}");
    assert!(checked.contains("cpl::pos(i - D_lo, D_len)"), "{checked}");
    let indented: String = body.lines().map(|l| format!("    {l}\n")).collect();
    let raw = cpp(&src(&format!("unchecked:\n{indented}")));
    assert!(!raw.contains("cpl::pos("), "{raw}");
    assert!(raw.contains("cpl::check_range(i, 0, 100"), "{raw}");
}

#[test]
fn a_const_as_a_type_argument_is_its_number() {
    let cpp = cpp("const MOD = 998244353\ntype Z = Mod<998244353>\nlet c = Comb<Mod<MOD>>()\nlet x: Z = Z(5)\nprint(c.choose(5, 2), x * x)\n");
    assert_eq!(cpp.matches("\nstruct Mod_").count(), 1, "{cpp}");
}

#[test]
fn an_index_of_the_arrays_own_domain_has_no_bounds_test() {
    // A `D` is one of `D`'s positions by construction; an `Int` is not.
    let src = "let n: Int = read()\ndomain D = 0..n\ndomain R = 0..n\nvar a: [D]Int = 0\nvar g: [R, D]Int = 0\n\
               var b: [0..n]Int = 0\nvar h: [0..n, 0..n]Int = 0\nlet k: Int = read()\n\
               for r in R:\n    for d in D:\n        a[d] += g[r, d]\n        g[r, d] = a[d]\n\
               b[k] = h[k, k]\nprint(a, g, b)\n";
    let out = cpp(src);
    assert!(out.contains("a[d] = cpl::iadd(a[d], g(r, d));"), "{out}");
    assert!(out.contains("g(r, d) = a[d];"), "{out}");
    assert!(out.contains("cpl::at(b, k) = h.ck(k, k);"), "{out}");
}

#[test]
fn read_int_scans_digits_without_a_bound_check_per_char() {
    let out = cpp("let n: Int = read()\nprint(n)\n");
    let start = out
        .find("inline ll read_int()")
        .expect("read_int is emitted");
    let read_int = &out[start..start + out[start..].find("\n}").unwrap()];
    assert!(read_int.contains("while (digit(in[in_at]))"), "{read_int}");
    assert!(!read_int.contains("in.size()"), "{read_int}");
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
    assert!(
        kept.contains("s = cpl::iadd(s, cpl::imul(cpl::at(a, 0), b[j]));"),
        "{kept}"
    );
    assert!(kept.contains("b[j] = cpl::iadd(b[j], 1);"), "{kept}");
    for body in [
        "        s += b[j]\n        if s > 9: b.pop()\n",
        "        s += b[j]\n        grow(mut b)\n",
        "        s += b[j]\n        b = [1]\n",
    ] {
        let out = dot(body);
        assert!(out.contains("cpl::at(b, j"), "{body}: {out}");
    }
}

#[test]
fn a_fn_is_declared_ahead_only_when_it_is_named_before_it_is_defined() {
    // A fn is written after the fns it calls, so only a cycle needs one.
    let cpp = cpp("fn first(n: Int) -> Int: second(n) + 1\n\
         fn second(n: Int) -> Int: n * 2\n\
         fn even(n: Int) -> Bool: n == 0 or odd(n - 1)\n\
         fn odd(n: Int) -> Bool: n != 0 and even(n - 1)\n\
         print(first(2), yes(even(4)))\n");
    let ahead = |sig: &str| cpp.contains(&format!("\n{sig};\n"));
    assert!(ahead("bool odd(ll n)") != ahead("bool even(ll n)"), "{cpp}");
    assert!(
        !ahead("ll first(ll n)") && !ahead("ll second(ll n)"),
        "{cpp}"
    );
    assert!(
        cpp.find("ll second(").unwrap() < cpp.find("ll first(").unwrap(),
        "{cpp}"
    );
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
    assert!(cpp.contains("mallopt(M_ARENA_MAX, 1);"), "{cpp}");
    let mut db = cpl_check::Session::new();
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
    let mut db = cpl_check::Session::new();
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
fn a_row_a_loop_never_moves_is_checked_once_per_loop() {
    let src = |inner: &str| {
        format!(
            "let n: Int = read()\nvar g = [[0 for _ in 0..n] for _ in 0..n]\nvar i: Int = read()\nfor j in 0..n:\n    g[i][j] += j\n{inner}print(g[0][0])\n"
        )
    };
    let once = cpp(&src(""));
    assert!(
        once.contains("auto row = cpl::lazy([&] { return &cpl::at(g, i); });"),
        "{once}"
    );
    assert!(once.contains("cpl::at(row(), j)"), "{once}");
    let moved = cpp(&src("").replace("g[i][j] += j", "g[i][j] += j\n    g.push([j])"));
    assert!(
        !moved.contains("cpl::lazy("),
        "a push onto g may move g[i]: {moved}"
    );
    let shifted = cpp(&src("").replace("g[i][j] += j", "g[i][j] += j\n    i = j"));
    assert!(
        !shifted.contains("cpl::lazy("),
        "the row index changes in the loop: {shifted}"
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
    assert!(cpp.contains("cpl::stable_sort_lt("), "{cpp}");
    assert!(!cpp.contains("cpl::sort_by("), "{cpp}");
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
        "print(f(x) + n, yes(big))\n",
    ));
    assert!(out.contains("static constexpr ll M = 1000000007;"), "{out}");
    assert!(out.contains("static constexpr ll n = 500000;"), "{out}");
    assert!(out.contains("static constexpr bool big = true;"), "{out}");
    assert!(
        !out.contains("  M = 1000000007;"),
        "constexpr M must not be stored again: {out}"
    );
    let stored_twice = cpp(concat!("var m = 5\n", "m += read<Int>()\n", "print(m)\n"));
    assert!(!stored_twice.contains("constexpr ll m"), "{stored_twice}");
    let closure_mut = cpp(concat!(
        "var c = 0\n",
        "let inc = () => c += 1\n",
        "inc()\n",
        "print(c)\n"
    ));
    assert!(!closure_mut.contains("constexpr ll c"), "{closure_mut}");
}

#[test]
fn a_char_pushed_from_a_conditional_is_cast_as_a_whole() {
    let body = body(
        "let s: Str = read()\nvar t = \"\"\nfor c in s: t += if c < 'm': 'a' else: 'z'\nprint(t)\n",
    );
    assert!(body.contains("t.push_back((char)("), "{body}");
}

#[test]
fn discarded_map_remove_emits_single_erase_without_map_get() {
    let out = body(concat!(
        "var m: Map<Int, Int> = [1: 10, 2: 20]\n",
        "let _ = m.remove(1)\n",
        "let v = m.remove(2)\n",
        "print(v ?? 0)\n",
    ));
    assert!(out.contains("m.erase(1);"), "expected direct erase: {out}");
    assert!(
        out.contains("cpl::map_take(m, 2)"),
        "expected map_take: {out}"
    );
    assert!(
        !out.contains("map_get"),
        "Map.remove must not emit map_get + erase: {out}"
    );
}

#[test]
fn unchecked_nested_index_assign_keeps_raw_cast() {
    let out = cpp(
        "fn scatter(mut sa: List<Int>, mut bkt: List<Int>, vec: List<Int>):
    with lint(unproven_list_index: allow)
    unchecked:
        for k in 0..vec.len:
            let c = vec[k]
            bkt[c] -= 1
            sa[bkt[c]] = k
var sa = [0, 0, 0]
var bkt = [3]
scatter(mut sa, mut bkt, [0, 0, 0])
print(sa[0])
",
    );
    let scatter_body = out
        .split("void scatter")
        .nth(1)
        .expect("scatter fn emitted");
    let scatter_body = scatter_body.split("\n}").next().unwrap();
    assert!(
        !scatter_body.contains("cpl::at"),
        "nested index assignment inside `unchecked:` should not emit `cpl::at`:\n{scatter_body}"
    );
}
#[test]
fn short_non_recursive_fns_are_declared_inline() {
    let out = cpp("fn sq(x: Int) -> Int = x * x
fn fact(n: Int) -> Int = if n <= 1: 1 else: n * fact(n - 1)
print(sq(7) + fact(5))
");
    assert!(out.contains("inline ll sq("), "{out}");
    assert!(
        out.contains("\nll fact("),
        "a recursive fn keeps a plain definition:\n{out}"
    );
    assert!(!out.contains("inline void solve("), "{out}");
}
#[test]
fn solve_only_globals_and_const_domain_lo_stay_internal_and_local() {
    let out = cpp("let n: Int = read()
domain D = 0..n
var acc = 0
for i in D:
    acc += i.ord
print(acc)
");
    assert!(out.contains("static ll D_len = 0;"), "{out}");
    assert!(
        !out.contains("D_lo = 0;"),
        "dead D_lo should be omitted:\n{out}"
    );
    let solve_start = out.find("void solve()").expect("solve");
    let before_solve = &out[..solve_start];
    let in_solve = &out[solve_start..];
    assert!(
        !before_solve.contains("ll acc = 0;"),
        "solve-only top-level var should not be a file-scope global:\n{before_solve}"
    );
    assert!(
        in_solve.contains("ll acc = 0;"),
        "solve-only top-level var should be local to solve():\n{in_solve}"
    );
}

#[test]
fn inline_hint_ignores_braces_for_tuple_swap_helper() {
    let out = cpp("fn gcd_ext(a: Int, b: Int) -> (Int, Int):
    var (r0, r1, x0, x1) = (a, b, 1, 0)
    while r1 != 0:
        let q = r0 / r1
        (r0, r1) = (r1, r0 - q * r1)
        (x0, x1) = (x1, x0 - q * x1)
    if r0 < 0: (-r0, -x0) else: (r0, x0)

let (a, b): (Int, Int) = read()
let (g, x) = gcd_ext(a, b)
print(g, x)
");
    assert!(
        out.contains("inline std::tuple<ll, ll> gcd_ext("),
        "short tuple-destructuring helper should receive inline hint:\n{out}"
    );
    assert!(
        out.contains("((uint64_t)a | (uint64_t)b) <= 0xFFFFFFFFULL"),
        "cpl::ediv should emit 32-bit unsigned fast path:\n{out}"
    );
}

#[test]
fn a_field_nothing_reads_is_dropped_with_its_stores_and_the_local_feeding_it() {
    let src = concat!(
        "domain V = 0..4\n",
        "struct Paths:\n",
        "    dist: [V]Int\n",
        "    back: [V]?(Int, V)\n",
        "    order: List<V>\n",
        "fn walk(k: Int) -> Paths:\n",
        "    var dist: [V]Int = 0\n",
        "    var back: [V]?(Int, V) = None\n",
        "    var order: List<V> = []\n",
        "    for i in V:\n",
        "        dist[i] = k\n",
        "        back[i] = (k, i)\n",
        "        order.push(i)\n",
        "    Paths(dist, back, order)\n",
        "let p = walk(read::<Int>())\n",
        "print(p.dist[V(3)], p.order.len)\n",
    );
    let out = cpp(src);
    let def = &out[out.find("struct Paths {").expect("Paths")..];
    let def = &def[..def.find("};").unwrap()];
    assert!(
        !def.contains("back") && def.contains("dist") && def.contains("order"),
        "{out}"
    );
    let walk = &out[out.find("inline Paths walk").expect("walk")..];
    let walk = &walk[..walk.find("\n}\n").unwrap()];
    assert!(
        !walk.contains(" back") && walk.contains("order.push_back"),
        "{out}"
    );
    let dropped = crate::explain_dropped_fields(&lower(src));
    assert_eq!(dropped, vec!["struct Paths.back".to_string()], "{out}");
}

#[test]
fn a_field_is_kept_when_a_store_can_trap_or_the_whole_value_is_seen() {
    let traps = concat!(
        "struct P:\n",
        "    a: Int\n",
        "    b: [0..2]Int\n",
        "fn make(k: Int) -> P:\n",
        "    var b: [0..2]Int = 0\n",
        "    b[k] = 1\n",
        "    P(k, b)\n",
        "print(make(read::<Int>()).a)\n",
    );
    let out = cpp(traps);
    assert!(
        out.contains("cpl::at(b, k"),
        "an index that can trap stays: {out}"
    );
    let compared = concat!(
        "struct Q:\n",
        "    a: Int\n",
        "    b: Int\n",
        "let k = read::<Int>()\n",
        "print(Q(k, 1) == Q(k, 2))\n",
    );
    assert!(crate::explain_dropped_fields(&lower(compared)).is_empty());
}

#[test]
fn char_array_reads_fill_in_one_pass() {
    let body = body("let n: 1..=10 = read()\nlet s: [0..n]Char = read()\nprint(s[0])\n");
    assert!(
        body.contains("cpl::read_chars(") && !body.contains("read_char()"),
        "a [D]Char read should fill the array in one pass:\n{body}"
    );
}

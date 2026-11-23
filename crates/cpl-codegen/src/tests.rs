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

fn lower(source: &str) -> cpl_ir::Program {
    let mut db = cpl_check::Database::new();
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
fn filter_map_sum_fuses_without_intermediate_vectors() {
    let body = body(
        "let a: List<Int> = [1, 2, 3, 4]\nprint(a.filter(x => (x & 1) == 0).map(x => x * 3 + 1).sum())\n",
    );
    assert!(
        !body.contains("push_back"),
        "filter.map.sum should fuse into one loop without intermediate vectors:\n{body}"
    );
}

#[test]
fn power_of_two_div_and_rem_use_shift_and_mask() {
    let body =
        body("var a: Int = read()\nprint(a / 2, a % 2, a / 8, a % 8)\na /= 2\na %= 4\nprint(a)\n");
    assert!(body.contains("a >> 1"), "{body}");
    assert!(body.contains("a & 1"), "{body}");
    assert!(body.contains("a >> 3"), "{body}");
    assert!(body.contains("a & 7"), "{body}");
    assert!(body.contains("a >>= 1;"), "{body}");
    assert!(body.contains("a &= 3;"), "{body}");
    assert!(
        !body.contains("cpl::divp") && !body.contains("cpl::modp"),
        "{body}"
    );
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

const TABLE: &str = "let n: 1..=100 = read()\ndomain V = 0..n\nlet w: [V]0..=1000 = read()\nlet any: Int = read()\nvar dp: [V]?Int = none\n";

fn table(rest: &str) -> String {
    let cpp = cpp(&format!("{TABLE}{rest}\nprint(dp[V.first!] ?? -1)\n"));
    decl(&cpp, "dp")
}

#[test]
fn an_optional_of_a_position_a_bool_or_a_char_is_one_scalar() {
    let cpp = cpp(
        "let n: 1..=9 = read()\ndomain V = 0..n\nvar p: [V]?V = none\nvar b: [V]?Bool = none\nvar c: [V]?Char = none\n\
         p[V.first!] = V.first!.next\nb[V.first!] = true\nc[V.first!] = 'a'\n\
         print(p[V.first!] ?? V.last!, yes(b[V.first!] ?? false), c[V.first!] ?? '-')\n",
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
    let cpp = cpp("var f: ?Bool = none\nvar c: ?Char = none\n(f ??= false) = true\nprint(yes(f ?? false), c ?? '-')\n");
    assert_eq!(decl(&cpp, "f"), "std::optional<bool>", "{cpp}");
    // A Char is still packed: only the Bool is opened in place.
    assert_eq!(decl(&cpp, "c"), "cpl::OptChar", "{cpp}");
}

#[test]
fn every_optional_int_is_one_int_with_int64_min_as_none() {
    const PACKED: &str = "std::vector<cpl::OptInt>";
    // Whatever is stored: unbounded input, sums, other tables.
    assert_eq!(table("dp[V.first!] = any"), PACKED);
    assert_eq!(table("dp[V.first!] = Int.MIN"), PACKED);
    assert_eq!(
        table("dp[V.first!] = -1\ndp[V.last!] = (dp[V.first!] ?? 0) * 2"),
        PACKED
    );
    assert_eq!(table("fn f() -> Int: any\ndp[V.first!] = f()"), PACKED);
    // Used as a whole value: passed, mutated through, looped over, copied.
    assert_eq!(
        table("fn f(xs: [V]?Int) -> Int: xs[V.first!] ?? 0\nprint(f(dp))"),
        PACKED
    );
    assert_eq!(
        table("fn f(mut xs: [V]?Int):\n    xs[V.first!] = 1\nf(mut dp)"),
        PACKED
    );
    assert_eq!(table("for x in dp: print(x ?? 0)"), PACKED);
    assert_eq!(
        table("let copy = dp.copy()\nprint(copy[V.first!] ?? 0)"),
        PACKED
    );
    // A list filled by `push`, a grid, a local, a parameter and a result.
    let cpp = cpp("fn half(x: ?Int) -> ?Int: x\nlet n: 0..=9 = read()\nvar xs: List<?Int> = []\nvar g: [n, n]?Int = none\nfor i in 0..n:\n    xs.push(none)\n    xs.push(i)\n    g[i, 0] = i\nlet best: ?Int = xs.pop()\nprint(xs.len, xs[0] ?? -1, g[0, 0] ?? -1, half(best) ?? -1)\n");
    assert_eq!(decl(&cpp, "xs"), "std::vector<cpl::OptInt>", "{cpp}");
    assert_eq!(decl(&cpp, "g"), "cpl::Grid<cpl::OptInt, 2>", "{cpp}");
    assert_eq!(decl(&cpp, "best"), "cpl::OptInt", "{cpp}");
    assert!(
        cpp.contains("cpl::OptInt half(const cpl::OptInt& x)"),
        "{cpp}"
    );
    assert!(
        cpp.contains("using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;"),
        "{cpp}"
    );
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
fn a_reversed_list_pays_only_for_the_view_parts_it_uses() {
    let read = "let n: Int = read()\nlet xs: List<Int> = [read() for _ in 0..n]\n";
    let size = |body: &str| cpp(&format!("{read}{body}")).len();
    let forward = size("var s = 0\nfor x in xs: s = s * 31 + x\nprint(s)\n");
    let backward = size("var s = 0\nfor x in xs.rev: s = s * 31 + x\nprint(s)\n");
    assert!(backward <= forward + 256, "{forward} -> {backward}");
    let plain = size("let ys = xs\nvar s = 0\nfor x in ys: s = s * 31 + x\nprint(s, ys[0])\n");
    let view = cpp(&format!(
        "{read}let ys = xs.rev\nvar s = 0\nfor x in ys: s = s * 31 + x\nprint(s, ys[0])\n"
    ));
    assert!(view.len() <= plain + 3200, "{plain} -> {}", view.len());
    assert!(
        !view.contains("list_concat") && !view.contains("list_slice"),
        "{view}"
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
fn a_bounded_optional_list_is_stored_in_four_bytes() {
    let code = cpp(
        "fn f(n: Int) -> List<?(0..2_147_483_648)>:\n    var a: List<?(0..2_147_483_648)> = List.filled(n, none)\n    a[0] = n\n    return a\nlet n: Int = read()\nlet b = f(n)\nvar c: List<?(0..100)> = [none]\nvar d: List<?Int> = [none]\nprint(b[0] ?? -1, c[0] ?? -1, d[0] ?? -1)\n",
    );
    let decl = |ty: &str| {
        code.lines()
            .filter(|l| l.trim_start().starts_with(&format!("std::vector<{ty}> ")))
            .count()
    };
    assert_eq!((decl("cpl::OptI32"), decl("cpl::OptInt")), (3, 1), "{code}");
    assert!(
        code.contains(", 0, 2147483647, \"0..2_147_483_648\")"),
        "{code}"
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
fn an_inclusive_loop_to_a_bound_below_int_max_is_a_plain_loop() {
    let open = cpp("let n: Int = read()\nfor i in 0..=n:\n    print(i)\n");
    assert!(open.contains("i_go"), "{open}");
    let len = cpp("let xs = [1, 2, 3]\nfor i in 0..=xs.len:\n    print(i)\n");
    assert!(len.contains("i <= "), "{len}");
    assert!(!len.contains("i_go"), "{len}");
    let tuple = cpp(
        "domain D = 0..5\nlet n: Int = read()\nlet (k, m) = (n, D.count)\nfor i in 1..=m - 1:\n    print(i, k)\n",
    );
    assert!(!tuple.contains("i_go"), "{tuple}");
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
    assert!(
        cpp.contains("zs = std::move(ys);"),
        "ys is not read again: {cpp}"
    );
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
        "print(last_of(2))\n",
        "let (_, vals) = compress([30, 10, 20, 10])\n",
        "print(vals.at(20))\n",
    ));
    assert!(
        cpp.contains("const std::vector<ll>& row = cpl::at(rows, r);"),
        "{cpp}"
    );
    // `vals.at(x)` searches the field where it is.
    assert!(!cpp.contains("= self.vals;"), "{cpp}");
}

#[test]
fn an_if_let_payload_nothing_changes_is_a_reference_not_a_copy() {
    let cpp = cpp(concat!(
        "fn total(n: Int, end: ?List<Int>) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        if let e = end: s += e[i % e.len]\n",
        "    s\n",
        "var best: ?List<Int> = some([1])\n",
        "fn swap() -> Int:\n",
        "    if let kept = best:\n",
        "        best = some([2, 3])\n",
        "        return kept.len\n",
        "    0\n",
        "print(total(5, some([1, 2, 3])), swap())\n",
    ));
    assert!(
        cpp.contains("const std::optional<std::vector<ll>>& t0 = end;"),
        "{cpp}"
    );
    assert!(cpp.contains("const std::vector<ll>& e = *t0;"), "{cpp}");
    // `swap` changes `best`: the optional is copied before the test.
    assert!(
        cpp.contains("std::optional<std::vector<ll>> t0 = best;"),
        "{cpp}"
    );
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
        "fn inside(i: Int) -> Int:\n",
        "    if i > 0:\n        let a = rows[i]\n        return a.len + size(i)\n",
        "    bump(i)\n    rows[i].len\n",
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

#[test]
fn an_optional_struct_with_an_int_field_has_no_flag() {
    let cpp = cpp(concat!(
        "struct Edge(to: Int, w: Int)\n",
        "struct Tag(c: Char)\n",
        "var best: ?Edge = none\n",
        "best = Edge(3, 4)\n",
        "var t: ?Tag = none\n",
        "t = Tag('a')\n",
        "print(yes(best == none), yes(t == none))\n",
    ));
    assert!(cpp.contains("cpl::OptField<&Edge::to> best"), "{cpp}");
    // A `Char` has no free value, so `?Tag` keeps its flag.
    assert!(cpp.contains("std::optional<Tag> t"), "{cpp}");
}

#[test]
fn an_optional_tuple_with_a_position_has_no_flag() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "var far: ?(depth: Int, child: V) = none\n",
        "far = (3, V.first!)\n",
        "var plain: ?(Int, Int) = none\n",
        "plain = (1, 2)\n",
        "print(far!.depth, plain!.0)\n",
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
         print(first(2), yes(even(4)))\n");
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
        explained
            .iter()
            .any(|e| matches!(e.mode, crate::ClosureMode::StaticDirect { .. })),
        "{explained:?}"
    );
    assert!(
        explained
            .iter()
            .all(|e| !matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
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
        explained
            .iter()
            .any(|e| e.mode == crate::ClosureMode::StaticTemplate),
        "{explained:?}"
    );
}

#[test]
fn non_escaping_closure_borrows_immutable_non_scalar_captures() {
    let out = cpp("struct Box(xs: List<Int>):
    fn find_first(self, target: Int) -> Int:
        (0..self.xs.len).bsearch(i => self.xs[i] >= target)
let b = Box([1, 3, 5, 7])
print(b.find_first(4))
");
    assert!(
        out.contains("[&self_2 = self"),
        "expected non-escaping closure passed to bsearch_first to capture `self` by reference:\n{out}"
    );
}

#[test]
fn unread_fn_fields_are_dropped_instead_of_std_function() {
    let src = concat!(
        "struct Acc:\n",
        "    total: Int\n",
        "    var hook: ?(Int -> Int)\n",
        "struct Tagged:\n",
        "    pick: ?(Int -> Int)\n",
        "let k = read::<Int>()\n",
        "let a = Acc(1, x => x * k)\n",
        "var b = Acc(2, none)\n",
        "b.hook = x => x + k\n",
        "let t = Tagged(x => x - k)\n",
        "print(a.total + b.total, yes(t.pick == none))\n",
    );
    let out = cpp(src);
    let acc = &out[out.find("struct Acc {").expect("Acc")..];
    assert!(!acc[..acc.find("};").unwrap()].contains("hook"), "{out}");
    assert!(
        out.contains("std::optional<std::function<ll(ll)>> pick;"),
        "{out}"
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let mode = |site: &str| &explained.iter().find(|e| e.site == site).expect(site).mode;
    assert_eq!(
        mode("struct Acc.hook"),
        &crate::ClosureMode::Unread,
        "{explained:?}"
    );
    assert!(
        matches!(mode("struct Tagged.pick"), crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("read")),
        "{explained:?}"
    );
}

#[test]
fn recursive_fn_threads_its_closure_param_as_a_template() {
    let src = concat!(
        "fn walk(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return 0\n",
        "    f(n) + walk(n - 1, f)\n",
        "fn even(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(0)\n",
        "    odd(n - 1, f)\n",
        "fn odd(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(1)\n",
        "    even(n - 1, f)\n",
        "fn wrap(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(0)\n",
        "    wrap(n - 1, x => f(x) + 1)\n",
        "let k: Int = read()\n",
        "print(walk(10, x => x * k), walk(5, x => x + k))\n",
        "print(even(7, x => x + k), odd(4, x => x * k))\n",
        "print(wrap(3, x => x + k))\n",
    );
    let out = cpp(src);
    assert!(out.contains("ll walk(ll n, F0&& f)"), "{out}");
    assert!(
        out.contains("ll even(ll n, F0&& f)") && out.contains("ll odd(ll n, F0&& f)"),
        "{out}"
    );
    assert!(
        out.contains("ll wrap(ll n, const std::function<ll(ll)>& f)"),
        "{out}"
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let dynamic: Vec<_> = explained
        .iter()
        .filter(|e| matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. }))
        .collect();
    assert_eq!(dynamic.len(), 1, "{explained:?}");
    assert!(
        matches!(&dynamic[0].mode, crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("different closure")),
        "{explained:?}"
    );
}

#[test]
fn recursive_local_closure_takes_itself_instead_of_std_function() {
    let src = concat!(
        "fn count(n: Int) -> Int:\n",
        "    var seen = 0\n",
        "    fn go(u: Int):\n",
        "        if u >= n: return\n",
        "        seen += 1\n",
        "        go(u * 2 + 1)\n",
        "        go(u * 2 + 2)\n",
        "    go(0)\n",
        "    seen\n",
        "print(count(10))\n",
    );
    let out = cpp(src);
    assert!(
        !out.contains("std::function") && !out.contains("<functional>"),
        "{out}"
    );
    assert!(out.contains("auto go = cpl::rec(["), "{out}");
    assert!(out.contains("](auto& go_2, ll u)"), "{out}");
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .all(|e| e.mode == crate::ClosureMode::StaticLocalAuto),
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
        explained
            .iter()
            .any(|e| e.mode == crate::ClosureMode::StaticLocalAuto),
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
fn a_field_ranged_by_a_shifted_const_narrows() {
    let cpp = cpp("const N = 1 << 30\nconst M = N | 1\nstruct Node(l: 0..N, r: 0..M)\nlet a = Node(1, 2)\nprint(a.l + a.r)\n");
    assert!(cpp.contains("uint32_t l = 0;"), "{cpp}");
    assert!(cpp.contains("uint32_t r = 0;"), "{cpp}");
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
    assert!(
        cpp.contains("uint32_t v = 0;"),
        "Mod<998244353> should narrow to uint32_t: {cpp}"
    );
    assert!(
        cpp.contains("int32_t w = 0;"),
        "signed range should narrow to int32_t: {cpp}"
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
    assert!(cpp.contains("std::stable_sort("), "{cpp}");
    assert!(!cpp.contains("cpl::sort_by("), "{cpp}");
}

#[test]
#[ignore = "spec/std/bitset.cp stores `w: List<U64>`: no inline Row for small domains yet"]
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
    assert!(
        cpp.contains("cpl::Row<unsigned long long, 5>::filled("),
        "{cpp}"
    );
    assert!(
        !cpp.contains("std::vector<unsigned long long>("),
        "Bitset.new() return must construct cpl::Row directly without temp std::vector: {cpp}"
    );
    assert!(cpp.contains("__builtin_popcountll("), "{cpp}");
    assert!(!cpp.contains("_bitand(a, b)"), "{cpp}");
}
#[test]
#[ignore = "spec/conformance/std-lazy/flip-action-on-a-segtree.cp"]
fn higher_order_combinator_devirtualizes_to_zero_std_function() {
    let cpp = cpp(concat!(
        "use std.lazy.{Flip, flip_action}\n",
        "domain I = 0..4\n",
        "let m = Monoid<Int>(0, (a, b) => a + b)\n",
        "let act = flip_action<Int>(m)\n",
        "var st = SegTree<I, Flip<Int>>(act.over, [I] i => if i.ord % 2 == 0: Flip(1, 0) else: Flip(0, 1), act: act)\n",
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
fn a_str_grows_in_place_when_appended_to() {
    let body =
        body("let s: Str = read()\nvar t = \"\"\nfor c in s: t += c\nprint(t, s[1..], s[..=1])\n");
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
        "fn ends<R: (u: Int, v: Int, ..)>(es: [E]R) -> Int = es[E(0)].u + es[E(2)].v\n",
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
    assert!(
        cpp.contains("Pack pack(std::vector<ll> xs, std::vector<ll> ys) {"),
        "{cpp}"
    );
    assert!(
        cpp.contains("return Pack{std::move(xs), std::move(ys)};"),
        "{cpp}"
    );
    assert!(
        cpp.contains("return ::prog::pack(std::move(a), std::move(b));"),
        "{cpp}"
    );
    assert!(
        cpp.contains("Pack twice(const std::vector<ll>& xs) {"),
        "named twice: copied, not sunk: {cpp}"
    );
    assert!(
        cpp.contains("::prog::twice(p.xs)"),
        "a global is not moved from: {cpp}"
    );
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
d[V.first!] += 5
print(d.sum())
");
    assert!(cpp.contains("d = std::move(t"), "{cpp}");
}

#[test]
fn map_keys_and_values_iterate_without_copying() {
    let out = body(concat!(
        "var m: HashMap<Int, Int> = [1: 10, 2: 20]\n",
        "var s = 0\n",
        "for k in m.keys: s += k\n",
        "print(s + m.values.sum())\n",
    ));
    assert!(
        !out.contains("std::vector"),
        "Map.keys and Map.values loops/folds must not allocate a vector: {out}"
    );
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
fn ranged_array_elements_narrow_to_32_bit_and_widen_on_plain_flow() {
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
        widened.contains("std::vector<ll> a"),
        "expected array passed to [V]Int param to widen to std::vector<ll>:\n{widened}"
    );
    assert!(
        !widened.contains("std::vector<uint32_t>"),
        "widened array must not emit uint32_t:\n{widened}"
    );
}

#[test]
fn unchecked_nested_index_assign_keeps_raw_cast() {
    let out = cpp("@allow(unproven_list_index)
fn scatter(mut sa: List<Int>, mut bkt: List<Int>, vec: List<Int>):
    unchecked:
        for k in 0..vec.len:
            let c = vec[k]
            bkt[c] -= 1
            sa[bkt[c]] = k
var sa = [0, 0, 0]
var bkt = [3]
scatter(mut sa, mut bkt, [0, 0, 0])
print(sa[0])
");
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
        grid.contains("cpl::Grid<int32_t, 2> s"),
        "expected [R, C]-100..=100 to emit cpl::Grid<int32_t, 2>:\n{grid}"
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
        tuples.contains("std::vector<std::tuple<uint32_t, int32_t>> pts"),
        "expected [V](0..=1_000_000, -100..=100) to emit std::vector<std::tuple<uint32_t, int32_t>>:\n{tuples}"
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
        widened.contains("cpl::Grid<ll, 2> g"),
        "expected 2D grid passed to [R, C]Int to widen:\n{widened}"
    );
    assert!(
        widened.contains("std::vector<std::vector<ll>> mat"),
        "expected nested array whose row flows to [E]Int to widen:\n{widened}"
    );
    assert!(
        widened.contains("std::vector<std::tuple<ll, ll>> pts"),
        "expected tuple array passed to [V](Int, Int) to widen:\n{widened}"
    );
}
#[test]
fn u32_struct_field_proves_nonneg_for_mod_mul() {
    let out = cpp("use std.mod.{Mod}
type Z = Mod<998_244_353>
let a: Int = read()
let b: Int = read()
print((Z(a) * Z(b)).v)
");
    let mul = out
        .split("\ninline ")
        .find(|s| s.contains("_mul("))
        .expect("Mod::mul emitted");
    assert!(
        mul.contains("% 998244353") && !mul.contains("cpl::modp"),
        "U32-narrowed Mod.v fields should prove non-negativity and avoid cpl::modp:\n{mul}"
    );
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
fn first_gt_and_last_lt_on_views_do_not_copy() {
    let out = cpp("use std.seq.{first_gt, last_lt}
let xs = [1, 2, 3, 3, 5, 7, 9]
let a = xs[1..6].first_gt(3)
let b = xs[1..6].last_lt(3)
let ps = [(1, 10), (2, 20), (3, 30), (3, 40), (5, 50)]
let c = ps[1..5].first_gt(3, by: p => p.0)
let d = ps[1..5].last_lt(3, by: p => p.0)
let s = \"abcde\"
let e = s[1..4].first_gt('c')
let f = s[1..4].last_lt('c')
print(a ?? -1, b ?? -1, c ?? -1, d ?? -1, e ?? -1, f ?? -1)
");
    let solve = out.split("void solve()").nth(1).expect("solve() emitted");
    assert!(
        !solve.contains("cpl::to_list(") && !solve.contains("cpl::to_str("),
        "first_gt/last_lt on slice views must not materialize a copy:\n{solve}"
    );
    assert!(
        solve.contains("cpl::list_slice(") && solve.contains("cpl::str_slice("),
        "first_gt/last_lt on slice views should search the O(1) view directly:\n{solve}"
    );
}

#[test]
fn seq_view_ops_pass_views_without_to_list_copy() {
    let out = cpp(
        "use std.seq.{merge_sorted, zip, compress, inversions, resized}
let xs = [1, 3, 5, 2, 4, 6]
let m = merge_sorted(xs[0..3], xs[3..6])
let z = zip(xs[0..3], xs[3..6])
let (C, c) = compress(xs[1..5])
let inv = inversions(xs[1..5])
let r = xs[1..4].resized(5, 0)
print(m.len, z.len, C.count, inv, r.len)
",
    );
    let solve = out.split("void solve()").nth(1).expect("solve() emitted");
    assert!(
        !solve.contains("cpl::to_list("),
        "generalized std.seq ops on slice views must not emit cpl::to_list:\n{solve}"
    );
}

#[test]
fn seq_take_is_o1_move_without_copy() {
    let out = cpp("use std.seq
var a = [1, 2, 3]
let b = seq.take_all(mut a)
print(a.len, b.len)
");
    let take_fn = out
        .split("inline std::vector<ll> m1_take_")
        .nth(1)
        .and_then(|s| s.split("void solve()").next())
        .expect("seq.take emitted");
    assert!(
        take_fn.contains("std::exchange("),
        "seq.take must move via std::exchange (O(1)):\n{take_fn}"
    );
    assert!(
        out.contains("#include <utility>\n"),
        "std::exchange needs <utility>; libstdc++ does not always pull it in:\n{out}"
    );
    assert!(
        !take_fn.contains(".resize(") && !take_fn.contains("cpl::copy("),
        "seq.take must not copy/truncate:\n{take_fn}"
    );
}

#[test]
fn multi_store_nonneg_var_drops_modp_and_sub_taints() {
    let out = cpp("let n: 1..=100 = read()
var s = 123
var x = 10
var acc = 0
for _ in 0..n:
    s = (s * 1103515245 + 12345) & 2147483647
    x -= 1
    acc += s % 1000 + x % 1000
print(acc)
");
    let body = out.split("void solve()").nth(1).expect("solve emitted");
    assert!(
        body.contains("s % 1000") && body.contains("cpl::modp(x, 1000)"),
        "bitmask-updated var should prove NonNeg while sub-updated var stays tainted:\n{body}"
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
fn a_local_read_for_the_last_time_moves_into_its_place() {
    let src = |after: &str| {
        format!(
            "let n: Int = read()\nvar dp = [0 for _ in 0..n]\nfor _ in 0..n:\n    let next = [dp[i] + i for i in 0..n]\n    dp = next\n{after}print(dp[0])\n"
        )
    };
    let moved = cpp(&src(""));
    assert!(moved.contains("dp = std::move(next);"), "{moved}");
    let read = cpp(&src("").replace(
        "    dp = next\n",
        "    dp = next.copy()\n    print(next[0])\n",
    ));
    assert!(
        !read.contains("dp = std::move(next);"),
        "next is read after the store: {read}"
    );
}

#[test]
fn a_local_memo_body_is_an_auto_lambda_and_its_wrapper_is_explained() {
    let src = "let n: Int = read()\nfn solve() -> Int:\n    @memo\n    fn fib(i: 0..=n) -> Int = if i < 2: i else: fib(i - 1) + fib(i - 2)\n    fib(n)\nprint(solve())\n";
    let out = cpp(src);
    assert!(out.contains("auto __memo_body"), "{out}");
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let wrapper = explained
        .iter()
        .find(|e| e.site == "fn solve local fib")
        .expect("wrapper");
    assert!(
        matches!(&wrapper.mode, crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("@memo")),
        "{explained:?}"
    );
}

#[test]
fn a_bound_method_value_is_its_lambda_not_a_std_function() {
    let src = concat!(
        "struct P:\n",
        "    w: Int\n",
        "    pub fn key(self, x: Int) -> Int = (x * w) % 7\n",
        "let p = P(read::<Int>())\n",
        "fn go(xs: List<Int>) -> List<Int> = xs.sorted(by: p.key)\n",
        "print(go([1, 2, 3, 4, 5]))\n",
    );
    let out = cpp(src);
    assert!(!out.contains("std::function"), "{out}");
}

#[test]
fn explain_names_closure_params_and_top_level_fn_vars() {
    let src = concat!(
        "let k = read::<Int>()\n",
        "let g = x => x + k\n",
        "let h = (f: Int -> Int, x: Int) => f(x) * 2\n",
        "print(h(g, 3), g(1))\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let reasons: Vec<String> = crate::explain_closures(&program)
        .into_iter()
        .filter_map(|e| match e.mode {
            crate::ClosureMode::DynamicStdFunction { reason } => Some(reason),
            _ => None,
        })
        .collect();
    assert!(
        reasons.iter().any(|r| r == "fn parameter of a closure"),
        "{reasons:?}"
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
        "    var back: [V]?(Int, V) = none\n",
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
fn optional_mut_param_does_not_emit_optional_of_reference() {
    let src = concat!(
        "domain V = 0..2\n",
        "fn f(seen: ?mut [V]Bool = None) -> Int:\n",
        "    0\n",
        "print(f())\n",
    );
    let out = cpp(src);
    assert!(!out.contains("std::optional<std::vector<char>&>"), "{out}");
    assert!(out.contains("std::optional<std::vector<char>>"), "{out}");
}

#[test]
fn a_guarded_step_of_a_global_counter_does_not_branch() {
    let cpp = cpp(
        "let a: [0..8]Int = read()\nvar seen: [10]Int = 0\nvar distinct = 0\n\
        fn add(i: Int):\n    if seen[a[i]] == 0: distinct += 1\n    seen[a[i]] += 1\n\
        fn rem(i: Int):\n    seen[a[i]] -= 1\n    if seen[a[i]] == 0: distinct -= 1\n\
        for i in 0..8: add(i)\nrem(0)\nprint(distinct)\n",
    );
    assert!(
        cpp.contains("distinct += (") && cpp.contains(") ? 1 : 0;"),
        "{cpp}"
    );
    assert!(cpp.contains("distinct -= ("), "{cpp}");
    assert!(!cpp.contains("distinct += 1;"), "{cpp}");
}

#[test]
fn a_returned_rekey_moves_its_local_list() {
    let cpp = cpp(concat!(
        "use std.strings.{z_function, prefix_function}\n",
        "var a = List.filled(4, 0)\n",
        "a[2] = 1\n",
        "let z = z_function(a)\n",
        "let p = prefix_function(a)\n",
        "print(z[1] + p[1])\n",
    ));
    assert!(cpp.contains("cpl::rekey_own(std::move(z"), "{cpp}");
    assert!(cpp.contains("cpl::rekey_own(std::move(cells"), "{cpp}");
}

#[test]
fn a_loop_over_one_field_that_writes_another_walks_it_in_place() {
    let cpp = cpp(concat!(
        "struct Bag:\n    w: List<Int>\n    n: Int\n",
        "    fn recount(mut self):\n        n = 0\n        for x in w: n += x\n",
        "    fn bump(mut self):\n        n += 1\n",
        "    fn refill(mut self):\n        for x in w: self.bump()\n",
        "var b = Bag(w: [1, 2, 3], n: 0)\n",
        "b.recount()\nb.refill()\n",
        "print(b.n + b.w.len)\n",
    ));
    let body = |name: &str| {
        let head = format!("void Bag_{name}(Bag& self) {{");
        let after = cpp.split(&head).nth(1).unwrap_or_default();
        after[..after.find("\n}").unwrap_or(after.len())].to_string()
    };
    assert!(body("recount").contains(": self.w)"), "{cpp}");
    assert!(body("refill").contains("= self.w;"), "{cpp}");
}

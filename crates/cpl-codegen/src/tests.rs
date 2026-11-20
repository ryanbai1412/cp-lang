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
        cpp.contains("inline ll modp(ll a, ll b)")
            && cpp.contains("return r + ((r >> 63) & b);"),
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
    assert!(hello.len() < 2000, "{} bytes for hello world", hello.len());
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
    let cpp = cpp(
        "var hm: HashMap<Int, Int> = []\nhm[42] = 7\nprint(hm.get(42) ?? 0)\n",
    );
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
    line.trim().split(&needle).next().unwrap_or_default().to_string()
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
    assert!(cpp.contains("using OptChar = Niche<u8, short, -1>;"), "{cpp}");
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
fn an_int_table_that_provably_never_holds_the_smallest_int_is_packed() {
    const PACKED: &str = "std::vector<cpl::OptInt>";
    // Literals, non-negative input, and sums that only grow.
    assert_eq!(table("dp[V.first] = 0\nfor v in V:\n    if let u = v + 1:\n        if let d = dp[v]: dp[u] max= d + w[v]"), PACKED);
    assert_eq!(table("dp[V.first] = -1\ndp[V.last] = Int.MAX"), PACKED);
    assert_eq!(table("for v in V: dp[v] = v.idx - w[v]"), PACKED);
    assert_eq!(table("for v in V: dp[v] = -(w[v] * 3) % 7"), PACKED);
    assert_eq!(table("for i in 0..n: dp[V.first] min= i * i"), PACKED);
    // A list filled by `push`, and a grid.
    let cpp = cpp("let n: 0..=9 = read()\nvar xs: List<?Int> = []\nvar g: [n, n]?Int = none\nfor i in 0..n:\n    xs.push(none)\n    xs.push(i)\n    g[i, 0] = i\nprint(xs.len, xs[0] ?? -1, g[0, 0] ?? -1)\n");
    assert_eq!(decl(&cpp, "xs"), "std::vector<cpl::OptInt>", "{cpp}");
    assert_eq!(decl(&cpp, "g"), "cpl::Grid<cpl::OptInt, 2>", "{cpp}");
    assert!(cpp.contains("xs.push_back(cpl::OptInt());"), "{cpp}");
    assert!(cpp.contains("xs.push_back(cpl::OptInt(i));"), "{cpp}");
    assert!(
        cpp.contains("using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;"),
        "{cpp}"
    );
}

#[test]
fn an_int_table_with_no_proof_stays_a_std_optional() {
    const PLAIN: &str = "std::vector<std::optional<ll>>";
    // The value itself.
    assert_eq!(table("dp[V.first] = Int.MIN"), PLAIN);
    // Input with no declared low end, and anything computed from it.
    assert_eq!(table("dp[V.first] = any"), PLAIN);
    assert_eq!(table("dp[V.first] = 0\ndp[V.last] = (dp[V.first] ?? 0) + any"), PLAIN);
    assert_eq!(table("let x = any + 1\ndp[V.first] = x"), PLAIN);
    // Two values above Int.MIN can still add up to it, and a difference
    // or a product of them can reach it.
    assert_eq!(table("dp[V.first] = -1\ndp[V.last] = (dp[V.first] ?? 0) + (dp[V.first] ?? 0)"), PLAIN);
    assert_eq!(table("dp[V.first] = -1\ndp[V.last] = (dp[V.first] ?? 0) - w[V.first]"), PLAIN);
    assert_eq!(table("dp[V.first] = -1\ndp[V.last] = (dp[V.first] ?? 0) * 2"), PLAIN);
    // What a fn returns or is handed is not followed.
    assert_eq!(table("fn f() -> Int: any\ndp[V.first] = f()"), PLAIN);
    assert_eq!(table("fn f(x: Int) -> Int:\n    dp[V.first] = x\n    0\nprint(f(any))"), PLAIN);
    // A variable stored twice counts for both stores.
    assert_eq!(table("var x = 0\nx = any\ndp[V.first] = x"), PLAIN);
    // Through another table.
    assert_eq!(table("var other: [V]?Int = none\nother[V.first] = any\ndp[V.first] = other[V.first]\nprint(other[V.first] ?? 0)"), PLAIN);
}

#[test]
fn an_int_table_used_as_a_whole_value_stays_a_std_optional() {
    const PLAIN: &str = "std::vector<std::optional<ll>>";
    assert_eq!(table("fn f(xs: [V]?Int) -> Int: xs[V.first] ?? 0\nprint(f(dp))"), PLAIN);
    assert_eq!(table("fn f(mut xs: [V]?Int):\n    xs[V.first] = 1\nf(mut dp)"), PLAIN);
    assert_eq!(table("for x in dp: print(x ?? 0)"), PLAIN);
    assert_eq!(table("print(dp)"), PLAIN);
    // An index that runs code is moved out of line with the table.
    assert_eq!(table("fn at() -> V: V.first\nprint(dp[at()] ?? 0)"), PLAIN);
    assert_eq!(table("let copy = dp\nprint(copy[V.first] ?? 0)"), PLAIN);
    // A closure holds its own copy or a reference.
    let cpp = cpp("fn f(n: Int) -> Int:\n    var xs: List<?Int> = []\n    xs.push(n % 2)\n    let get = i => xs[i] ?? 0\n    get(0)\nprint(f(3))\n");
    assert_eq!(decl(&cpp, "xs"), PLAIN, "{cpp}");
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
        cpp.contains("\nstd::tuple<int, std::vector<int>> best(int n) {\n    cpl_start();"),
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

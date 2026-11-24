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
mod passes;
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
    // About 1 kB of it runs the program on a thread with the judge's stack,
    // and about 300 B is the output buffer.
    assert!(hello.len() < 2850, "{} bytes for hello world", hello.len());
}

#[test]
fn a_printed_number_is_written_in_place_into_the_output_buffer() {
    let cpp = cpp("let n: Int = read()\nprint(n)\n");
    let put = &cpp[cpp.find("inline void put_int(").unwrap()..];
    let put = &put[..put.find("\n}").unwrap()];
    assert!(
        put.contains("out.room(22)") && !put.contains("append"),
        "{put}"
    );
    assert!(!cpp.contains("std::string out"), "{cpp}");
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
fn an_array_over_a_constant_domain_is_a_std_array_in_every_position() {
    let source = "\
domain K = 0..3
struct Sum:
    c: [K]Int
fn join(a: [K]Int, b: [K]Int) -> [K]Int = [K] k => a[k] + b[k]
fn pick(p: ([K]Int, Int)) -> ?[K]Int = if p.1 > 0: Some(p.0) else: None
let x: [K]Int = [K] k => k.ord
let s = Sum(join(x, x))
let o = pick((s.c, 1))
print(join(x, x)[K(1)], s.c[K(2)], (o ?? x)[K(0)])
";
    let out = cpp(source);
    let body = &out[out.find("namespace prog {").expect("prog")..];
    assert!(
        body.contains(
            "std::array<ll, 3> join(const std::array<ll, 3>& a, const std::array<ll, 3>& b)"
        ) && !body.contains("std::vector<"),
        "params, returns, fields, tuples and optionals all hold the array inline:\n{body}"
    );
}

#[test]
fn extend_borrows_its_source_and_push_takes_a_dying_local() {
    let body = body(
        "let n: Int = read()\nvar stack: List<(val: Int, xs: List<Int>)> = []\nfor i in 0..n:\n    var xs = [i]\n    if stack.len > 0: xs.extend(stack[stack.len - 1].xs)\n    stack.push((i, xs))\nvar again: List<Int> = [1]\nagain.extend(again)\nprint(stack.len, again.len)\n",
    );
    assert!(body.contains("const auto& t0 = std::get<1>("), "{body}");
    assert!(body.contains("{i, std::move(xs)}"), "{body}");
    // A list grown by its own items reads a copy: inserting a vector's own range is undefined.
    assert!(body.contains("auto t2 = again;"), "{body}");
}

#[test]
fn a_loop_over_a_global_the_fn_never_writes_walks_it_in_place() {
    let cpp = cpp(
        "domain Val = 0..100\nvar divisors: [Val]List<Val> = [Val] _ => []\nvar a: [Val]Val = [Val] v => v\nvar hits: [Val]Int = [Val] _ => 0\nfn touch(p: Val):\n    for d in divisors[a[p]]:\n        hits[d] += 1\nfor v in Val:\n    divisors[v].push(v)\n    touch(v)\nprint(hits[Val(1)])\n",
    );
    let touch = &cpp[cpp.find("inline void touch").unwrap()..];
    let touch = &touch[..touch.find("\n}\n").unwrap()];
    assert!(touch.contains(": divisors[a[p]])"), "{touch}");
    assert!(!touch.contains("std::vector"), "{touch}");
}

#[test]
fn list_insert_and_remove_move_the_tail_once() {
    let body = body("var a = [1, 3]\na.insert(1, 2)\nlet y = a.remove(0)\nprint(a, y)\n");
    assert!(body.contains("cpl::list_insert(a, "), "{body}");
    assert!(body.contains("cpl::list_remove(a, "), "{body}");
    assert!(!body.contains("swap_at"), "{body}");
}

#[test]
fn an_optional_read_checks_for_the_end_of_input_first() {
    let body =
        body("var xs = List<Int>()\nwhile let x = read<?Int>():\n    xs.push(x)\nprint(xs.len)\n");
    assert!(
        body.contains("if (cpl::at_eof()) return cpl::OptInt();"),
        "{body}"
    );
}

#[test]
fn a_read_from_a_string_swaps_the_input_in_a_scope() {
    let cpp = cpp("let x = \"4\".parse<Int>()!\nprint(x)\n");
    assert!(cpp.contains("cpl::parse_scope cpl_scope("), "{cpp}");
    assert!(cpp.contains("catch (const cpl::parse_failed&)"), "{cpp}");
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
fn a_table_key_hashes_over_its_cells() {
    // A `Map` keyed by a board of optional enum cells (in a tuple) or a fixed row: CpHash walks
    // a Grid's sizes and cells and a Row's cells, so the key type has a hash in C++.
    let cpp = cpp(
        "domain R = 0..2\nenum P = J | D\nvar m: Map<([R, R]?P, P), Int> = Map()\nvar b: [R, R]?P = None\nm[(b, J)] = 1\nprint(m.len)\n",
    );
    assert!(
        cpp.contains("HashMap<std::tuple<cpl::Grid<std::optional<P>, 2>, P>"),
        "{cpp}"
    );
    assert!(
        cpp.contains("decltype(g.d.data(), g.n[0], size_t())"),
        "{cpp}"
    );
    assert!(cpp.contains("std::is_enum_v<I>"), "{cpp}");
    assert!(
        cpp.contains("decltype(r.vec(), r.a.data(), size_t())"),
        "{cpp}"
    );
}

#[test]
fn a_map_of_ranged_keys_and_values_stores_them_narrow() {
    let narrow = cpp(concat!(
        "let n: 1..=100 = read()\n",
        "var c: Counter<-5..=1000, 0..=4000000> = Counter()\n",
        "for i in 0..n: c.add(i, 1)\n",
        "print(c[3])\n",
    ));
    assert!(
        narrow.contains("cpl::HashMap<int16_t, uint32_t, cpl::CpHash> m;"),
        "{narrow}"
    );
    // `+=` sums wide and checks the count against its range: the cell's width can't wrap it back in.
    assert!(
        narrow.contains("cpl::fit<uint32_t>(cpl::iadd(ll("),
        "{narrow}"
    );
    assert!(narrow.contains("0, 4000000, \"0..=4000000\")"), "{narrow}");
    // A map given whole from elsewhere keeps its stored width.
    let wide = cpp(concat!(
        "struct Hold:\n    m: HashMap<0..=9, 0..=9>\n",
        "var m: HashMap<Int, Int> = []\nm[1] = 2\n",
        "let h = Hold(m)\n",
        "print(h.m.len)\n",
    ));
    assert!(
        wide.contains("cpl::HashMap<ll, ll, cpl::CpHash> m;"),
        "{wide}"
    );
}

#[test]
fn a_narrow_map_field_stays_narrow_when_given_an_empty_or_narrow_map() {
    let src = |o: &str| {
        format!(
            "struct Hold:\n    var m: HashMap<0..=9, 0..=9>\n    var o: {o}\nvar h = Hold([], [])\nh.m[1] = 2\nh.o = h.m\nh.m = []\nprint(h.m.len, h.o.len)\n"
        )
    };
    let same = cpp(&src("HashMap<0..=9, 0..=9>"));
    assert_eq!(
        same.matches("cpl::HashMap<uint8_t, uint8_t, cpl::CpHash> ")
            .count(),
        2,
        "{same}"
    );
    assert!(
        same.contains("h.m = cpl::HashMap<uint8_t, uint8_t, cpl::CpHash>()"),
        "{same}"
    );
    // Copied into a map of another layout, both keep their full width.
    let other = cpp(&src("HashMap<0..=9, 0..=999>"));
    assert!(!other.contains("uint8_t, uint8_t"), "{other}");
}

#[test]
fn hash_map_uses_salted_splitmix64_cphash() {
    let cpp = cpp("var hm: HashMap<Int, Int> = []\nhm[42] = 7\nprint(hm.get(42) ?? 0)\n");
    assert!(cpp.contains("struct CpHash"), "{cpp}");
    assert!(cpp.contains("splitmix64"), "{cpp}");
    assert!(cpp.contains("steady_clock"), "{cpp}");
    // The salt is read once at start-up, and an integer key is one multiply by it.
    assert!(
        cpp.contains("static uint64_t salt() { return hash_salt; }"),
        "{cpp}"
    );
    assert!(
        cpp.contains("return (size_t)((uint64_t)x * (salt() | 1));"),
        "{cpp}"
    );
    assert!(
        cpp.contains("(uint8_t)(0x80 | ((h >> 32) & 0x7f))"),
        "{cpp}"
    );
    // Fingerprinted slots at a load of at most 3/4, sized exactly, freeing the old table as a rehash walks it.
    // A load of 2/3 measured no faster on bench/containers/hash_reserve_probe and slower on
    // hash_str_grow and hash_antihash, which grow 1.4x the memory.
    assert!(
        cpp.contains("n + (n + 2) / 3") && cpp.contains("release_pages"),
        "{cpp}"
    );
    // The state bytes are an array of their own, which stays in cache while the entries miss:
    // a state packed into each Int entry pads it to 24 B, and a group's 8 states before its 8
    // entries measured slower on hash_reserve_probe.
    assert!(
        cpp.contains("uint8_t* st = nullptr; E* e = nullptr;"),
        "{cpp}"
    );
    // Whatever the entry, so a Map<Str, _> doesn't hold both tables whole.
    assert!(
        !cpp.contains("is_trivially_destructible_v<E>) if (i + 1 - done"),
        "{cpp}"
    );
    assert!(!cpp.contains("2 * (n + 1)"), "{cpp}");
    assert!(cpp.len() < 65536, "{} bytes", cpp.len());
}

#[test]
fn a_one_field_struct_key_hashes_as_its_field() {
    let cpp = cpp("struct H(v: Int)\nvar hm = HashMap<H, Int>()\nhm[H(4)] = 7\nprint(hm[H(4)])\n");
    assert!(
        cpp.contains("if constexpr (I == 0 && std::tuple_size<T>::value == 1) return (uint64_t)(*this)(std::get<0>(t));"),
        "{cpp}"
    );
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
        cpp.contains("return cpl::Judge<std::vector<ll>, std::vector<int>>::from(::ask(cpl::Judge<ll, int>::to(i)));"),
        "{cpp}"
    );
    // The fn the grader calls, in its types, and no `main` beside the grader's.
    assert!(
        cpp.contains("\nstd::pair<int, std::vector<int>> best(int n) {\n  cpl_start();"),
        "{cpp}"
    );
    assert!(!cpp.contains("int main()"), "{cpp}");
}

#[test]
fn a_grader_fn_crosses_in_the_narrowest_type_its_declared_range_fits() {
    // IOI 2025 souvenirs: `std::pair<std::vector<int>, long long>
    // transaction(long long M)` and `void buy_souvenirs(int N, long long P0)`.
    let out = cpp(
        "@grader fn transaction(m: 1..=1e15) -> (List<0..100>, 0..=1e15)\n\
         @grader\nfn buy_souvenirs(count: 2..=100, p0: 1..=1e15):\n    let (got, back) = transaction(p0 - 1)\n    print(got.len, back, count)\n",
    );
    let runtime = out.find("namespace cpl").expect("a runtime");
    assert!(
        out[..runtime].contains("std::pair<std::vector<int>, long long> transaction(long long m);"),
        "{out}"
    );
    assert!(
        out.contains("\nvoid buy_souvenirs(int count, long long p0) {"),
        "{out}"
    );
    assert!(
        out.contains("cpl::Judge<std::tuple<std::vector<ll>, ll>, std::pair<std::vector<int>, long long>>::from"),
        "{out}"
    );
    let wide = cpp("@grader\nfn f(a: (Int, Int, 0..=1e12)) -> Int = a.0\n");
    assert!(
        wide.contains("\nint f(std::tuple<int, int, long long> a) {"),
        "{wide}"
    );
    let widths = cpp(
        "@grader\nfn f(a: -2_147_483_648..=2_147_483_647, b: -9e18..=9e18, c: 0..=2_147_483_648) -> Int = 0\n",
    );
    assert!(
        widths.contains("\nint f(int a, long long b, long long c) {"),
        "{widths}"
    );
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
fn a_key_tested_and_then_stored_through_a_path_is_looked_up_once() {
    let code = cpp(concat!(
        "struct B(h: Int, len: Int)\n",
        "var maps: [1..=3]HashMap<Int, Int> = []\n",
        "maps[2][5] = 0\n",
        "let b = B(5, 2)\n",
        "for i in 0..4:\n",
        "    if maps[b.len].contains(b.h): maps[b.len][b.h] += i\n",
        "print(maps[2][5])\n",
    ));
    assert!(code.contains("cpl::map_slot("), "{code}");
    assert!(!code.contains(".count(b.h)"), "{code}");
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
fn a_walk_over_an_arrays_items_checks_no_position() {
    let out = cpp(r#"
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for (i, x) in a.items:
        s += x * Int(a.cut_after(i)) + Int(a.cut_before(i))
    s
let n: Int = read()
let v: List<Int> = [0..n] i => i
print(total(v), v.scan(min).prefix(1))
"#);
    let walk = &out[out.find("total").unwrap()..];
    assert!(!walk.contains("cpl::pos("), "{walk}");
    assert!(!walk.contains("cut_beside("), "{walk}");
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
    assert!(
        out.contains("a[d] = cpl::iadd(a[d], g_row[(size_t)(d)]);"),
        "{out}"
    );
    assert!(out.contains("g_row[(size_t)(d)] = a[d];"), "{out}");
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
fn a_domain_read_takes_the_unsigned_scan() {
    let cpp = cpp("let n: 1..=9 = read()\ndomain V = 1..=n\nlet v: V = read()\nprint(v)\n");
    assert!(cpp.contains("cpl::read_pos(V_lo, V_len)"), "{cpp}");
    assert!(!cpp.contains("cpl::pos(cpl::read_int()"), "{cpp}");
}

#[test]
fn the_input_buffer_drops_what_was_read() {
    let cpp = cpp("let n: Int = read()\nprint(n)\n");
    assert!(cpp.contains("in.erase(0, keep)"), "{cpp}");
    assert!(
        cpp.contains("if (in_at + k >= in.size() && !more())"),
        "{cpp}"
    );
}

#[test]
fn a_printed_sequence_streams_into_out() {
    for src in [
        "let n: 1..=9 = read()\nprint(0..n)\n",
        "let n: 1..=9 = read()\nprint(i for i in 1..=n)\n",
        "let n: 1..=9 = read()\nlet xs = [i * 7 - n for i in 0..n]\nprint(xs, sep: \"\\n\")\n",
    ] {
        let cpp = cpp(src);
        let body = &cpp[cpp.find("void solve()").unwrap()..];
        assert!(!body.contains("Capture"), "{body}");
        assert!(!body.contains("-> std::vector<ll>"), "{body}");
    }
    let put = cpp("let s: Str = read()\nprint(s, 7)\n");
    assert!(
        put.contains("out.append(s.data(), s.size()); spill(); }"),
        "{put}"
    );
    let put_int = &put[put.find("inline void put_int").unwrap()..];
    assert!(
        put_int[..put_int.find("\n}").unwrap()].contains("spill();"),
        "{put}"
    );
}

#[test]
fn a_program_that_sorts_keeps_its_freed_heap() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "for r in 0..n:\n",
        "    var v = [0..n] i => (i * r) % 7\n",
        "    v.sort(by: x => -x)\n",
        "    print(v[0])\n",
    ));
    assert!(cpp.contains("#include <malloc.h>\n"), "{cpp}");
    assert!(cpp.contains("mallopt(M_TRIM_THRESHOLD, -1);"), "{cpp}");
    // A big buffer stays mapped: on the untrimmed heap, the blocks a growing
    // list frees pile up (ioi08p1 went from 61 to 103 MB).
    assert!(!cpp.contains("M_MMAP_THRESHOLD"), "{cpp}");
}

#[test]
fn the_program_runs_with_the_profile_stack() {
    let source = "let n: Int = read()\nprint(n)\n";
    let cpp = cpp(source);
    assert!(cpp.contains("Runs with a 1024 MB stack."), "{cpp}");
    assert!(cpp.contains("setcontext(&u)"), "{cpp}");
    assert!(cpp.contains("static void cpl_main() {"), "{cpp}");
    assert!(cpp.contains("for (size_t n = (size_t)1024 << 20;"), "{cpp}");
    assert!(
        cpp.contains("pthread_attr_setstacksize") && !cpp.contains("CreateThread"),
        "{cpp}"
    );
    // One thread, so glibc keeps malloc's single-thread fast path; no sort,
    // so no heap to keep.
    assert!(!cpp.contains("mallopt"), "{cpp}");
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
    // ASan follows a pthread's stack, so a sanitized build takes that path.
    assert!(
        cpp.contains("#if defined(__GLIBC__) && !defined(__SANITIZE_ADDRESS__)"),
        "{cpp}"
    );
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
fn widened_ints_multiply_in_i128_unchecked() {
    let cpp = cpp(concat!(
        "let (a, b): (Int, Int) = read()\n",
        "let c = I128(a) * I128(b)\n",
        "print(c * I128(a))\n",
    ));
    assert!(cpp.contains("(__int128)a * (__int128)b"), "{cpp}");
    assert!(cpp.contains("cpl::imul128(c, (__int128)a)"), "{cpp}");
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
    assert!(
        !uncompared.contains("cpl_tie"),
        "nothing reads an uncompared struct's tie: {uncompared}"
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
fn a_fn_whose_name_ends_a_runtime_helper_is_still_inline() {
    let out = cpp("let n: Int = read()
fn add(x: Int, best: Int) -> Int = max(best, x * (n + 1) + x)
print(add(n, 0))
");
    assert!(out.contains("cpl::iadd("), "{out}");
    assert!(out.contains("inline ll add("), "{out}");
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

#[test]
fn a_local_fn_guards_the_runtime_domains_it_declares() {
    let out = cpp("for t in 0..2:
    fn depth(n: Int) -> Int:
        if n == 0: return 0
        domain D = 0..n
        let inner = [D] d => if d.ord == 0: depth(n - 1) else: 0
        D.count * 10 + inner[D(0)]
    print(depth(3 + t))
");
    assert!(out.contains("cpl::DomGuard"), "{out}");
}

#[test]
fn an_element_a_mut_call_may_move_is_copied_whatever_its_index() {
    let out = cpp("struct T<K>:
    var keys: List<K>
    fn grow(mut self, x: K) -> Int:
        for _ in 0..1000: keys.push(x)
        keys.len
    fn dup(mut self, i: Int) -> Int = self.grow(keys[i * 2])
var t = T<(Int, Int)>([(1, 2)])
print(t.dup(0))
");
    assert!(out.contains("_grow(self, std::tuple<ll, ll>("), "{out}");
}

#[test]
fn a_dsuof_merge_finds_each_root_once_and_stores_ordinals_unchecked() {
    let out = cpp("domain V = 0..8
var d = DsuOf<V, Int>(v => v.ord, (a, b) => a + b)
let a: Int = read()
print(yes(d.unite(V.nth(a), V.nth(a + 1))), d.get(V.nth(a)))
");
    let start = out.find("_merge(").expect("merge is emitted");
    let body = &out[start..start + out[start..].find("\n}\n").expect("merge ends")];
    assert_eq!(body.matches("_find_root(self").count(), 2, "{body}");
    assert_eq!(body.matches("check_range(").count(), 1, "{body}");
    assert!(body.contains("self.p[(ll)y] = int32_t((ll)x);"), "{body}");
    let unite = &out[out.find("_unite(").expect("unite is emitted")..];
    assert!(
        !unite[..unite.find("\n}\n").unwrap()].contains("find"),
        "{out}"
    );
}

fn fn_text(cpp: &str, head: &str) -> String {
    let start = cpp
        .find(head)
        .unwrap_or_else(|| panic!("no `{head}`:\n{cpp}"));
    let len = cpp[start..].find("\n}\n").expect("the fn ends");
    cpp[start..start + len].to_string()
}

#[test]
fn a_recursive_fns_branch_past_its_last_call_has_its_own_frame() {
    let out = cpp(concat!(
        "fn ends(n: Int) -> List<Int>:\n",
        "    if n == 0: return [1]\n",
        "    var res = ends(n - 1)\n",
        "    res.push(0)\n",
        "    if n % 2 == 0:\n",
        "        for i in 1..res.len: res[i] += res[i - 1]\n",
        "        res\n",
        "    else:\n",
        "        let w = n + 1\n",
        "        res.map(e => e * w)\n",
        "fn sum_to(n: Int) -> Int:\n",
        "    if n == 0: return 0\n",
        "    var s = 0\n",
        "    for i in 0..n: s += i\n",
        "    s + sum_to(n - 1)\n",
        "fn flat(n: Int) -> List<Int> = [i * 2 for i in 0..n]\n",
        "print(ends(5).sum(), sum_to(6), flat(3).len)\n",
    ));
    let ends = fn_text(&out, "std::vector<ll> ends(ll n) {");
    assert_eq!(
        ends.matches("__attribute__((noinline))").count(),
        1,
        "{ends}"
    );
    let (inline, cold) = ends.split_once("__attribute__((noinline))").unwrap();
    assert!(
        inline.contains("ends(cpl::isub(n, 1))") && !cold.contains("ends("),
        "{ends}"
    );
    assert!(
        cold.contains("ll w = ") && !inline.contains("ll w = "),
        "{ends}"
    );
    assert!(
        inline.contains("return res;"),
        "the branch returning `res` stays inline:\n{ends}"
    );
    assert!(
        !fn_text(&out, "ll sum_to(ll n) {").contains("noinline"),
        "a loop before the recursive call runs on every frame anyway:\n{out}"
    );
    assert!(!fn_text(&out, "flat(ll n) {").contains("noinline"), "{out}");
}

#[test]
fn a_cold_tails_sort_key_parameter_is_bound_inside_it() {
    let out = cpp(concat!(
        "fn go(n: Int) -> F64:\n",
        "    if n == 0: return 0.0\n",
        "    var xs: List<F64> = []\n",
        "    for i in 0..n: xs.push(F64(i) + go(n - 1))\n",
        "    let best = xs.sorted(by: x => desc(x)).take(2)\n",
        "    best.sum()\n",
        "print(go(4))\n",
    ));
    let go = fn_text(&out, "double go(ll n) {");
    let (_, cold) = go.split_once("__attribute__((noinline))").expect(&go);
    let call = cold.rsplit_once("}(").map_or("", |(_, args)| args);
    assert!(!call.contains("x_"), "the key's `x` is not an input:\n{go}");
}

#[test]
fn a_record_spread_of_a_local_reads_its_fields_without_a_lambda() {
    let cpp = cpp(concat!(
        "let n: 1..=3 = read()\n",
        "let rs = [(u: n, v: n + 1, w: 5), (u: 2, v: 1, w: 7)]\n",
        "var out = List<(to: Int, e: Int, w: Int)>()\n",
        "for e in 0..rs.len:\n",
        "    let r = rs[e]\n",
        "    out.push((to: r.v, e: e, ..r without (u, v)))\n",
        "print(out)\n",
    ));
    assert!(!cpp.contains("[&]()"), "{cpp}");
}

#[test]
fn a_cold_branch_keeps_the_instances_of_a_generic_fn_one_template() {
    let source = concat!(
        "fn below<T: Ord>(xs: List<T>, n: Int, x: T) -> Int:\n",
        "    if n == 0: return 0\n",
        "    let r = below(xs, n - 1, x)\n",
        "    let y = xs[n % xs.len]\n",
        "    let up = if y < x: 1 else: 0\n",
        "    r + up\n",
        "print(below([3, 1, 2], 4, 2), below([(1, 2), (0, 5)], 3, (1, 0)))\n",
    );
    let out = cpp(source);
    let heads: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("__attribute__((noinline))"))
        .collect();
    assert_eq!(heads.len(), 2, "{out}");
    assert_eq!(
        heads[0], heads[1],
        "the Int and the tuple instance differ:\n{out}"
    );
    assert!(heads[0].contains("auto&& x"), "{out}");
    let program = lower(source);
    let fitted = crate::emit_aiming(
        &program,
        "t.cp",
        source,
        (1, 1 << 20),
        &crate::Target::default(),
    )
    .unwrap_or_else(|f| f);
    assert!(!fitted.folded.is_empty(), "{}", fitted.cpp);
    assert_eq!(fitted.cpp.matches("noinline").count(), 1, "{}", fitted.cpp);
}

#[test]
fn the_sizes_of_a_file_with_folded_instances_add_up_to_it() {
    let source = concat!(
        "fn count_below<T: Ord>(xs: List<T>, cap: T) -> Int:\n",
        "    var n = 0\n",
        "    for x in xs:\n",
        "        if x < cap: n += 1\n",
        "    n\n",
        "fn both_counts_of_the_lists() -> Int:\n",
        "    count_below([3, 1, 2], 2) + count_below([(1, 2), (0, 5)], (1, 0))\n",
        "print(both_counts_of_the_lists())\n",
    );
    let program = lower(source);
    let target = crate::Target::default();
    let fitted =
        crate::emit_aiming(&program, "t.cp", source, (1, 1 << 20), &target).unwrap_or_else(|f| f);
    assert!(!fitted.folded.is_empty(), "{}", fitted.cpp);
    let sizes = crate::sizes(
        &program,
        "t.cp",
        fitted.header.then_some(source),
        &target,
        &fitted,
    );
    let total: usize = sizes.iter().map(|size| size.bytes).sum();
    assert_eq!(total, fitted.cpp.len(), "{sizes:?}\n{}", fitted.cpp);
    // The caller's calls are to the template, `Q0<const std::vector<ll>&,ll>(..)`.
    let caller = &fitted.names["both_counts_of_the_lists"];
    let at = fitted.cpp.find(&format!("inline ll {caller}(")).unwrap();
    let def = &fitted.cpp[at..at + fitted.cpp[at..].find("\n}\n").unwrap() + 3];
    let measured = sizes
        .iter()
        .find(|size| size.name == "both_counts_of_the_lists");
    assert_eq!(
        measured.map(|size| size.bytes),
        Some(def.len()),
        "{sizes:?}\n{def}"
    );
}

#[test]
fn a_table_comprehension_writes_cells_nothing_filled_first() {
    let cpp = cpp(concat!(
        "let (n, m): (Int, Int) = read()\n",
        "let t = [0..n, 0..m] (i, j) => i * j\n",
        "let u = [0..n] i => i + 1\n",
        "var v: [0..n]Int = 0\n",
        "for i in 0..n - 1: v[i] = i\n",
        "print(t[n - 1, m - 1] + u[0] + v[n - 1])\n",
    ));
    assert!(cpp.contains("cpl::blank_grid<ll, 2>("), "{cpp}");
    assert!(cpp.contains("cpl::blank<ll>("), "{cpp}");
    assert_eq!(cpp.matches("cpl::blank<ll>(").count(), 1, "{cpp}");
}

#[test]
fn a_cell_the_round_before_stored_is_read_from_a_local() {
    let src = |body: &str| {
        format!(
            "let n: 2..=9 = read()\ndomain I = 0..n\nvar s: [I]Int = 0\nfor i in I:\n    s[i] = read()\n\
             var inner: [I]0..=n = 0\nfor i in I.rev:\n    var need: [I]0..=n = 0\n    for j in i.after:\n{body}\
             \x20   inner = need\nprint(inner[I.last!])\n"
        )
    };
    let dp = "        need[j] = if s[i] == s[j]: inner[j.prev!] else: min(1 + min(inner[j], need[j.prev!]), n)\n";
    let carried = cpp(&src(dp));
    assert!(carried.contains("ll carry = 0;"), "{carried}");
    assert!(carried.contains("need[j] = uint8_t(carry);"), "{carried}");
    assert!(
        carried.contains("std::min<ll>(ll(inner[j]), carry)"),
        "{carried}"
    );
    assert!(!carried.contains("need[j - 1]"), "{carried}");
    let twice = cpp(&src(&format!("{dp}        need[i] = 0\n")));
    assert!(!twice.contains("carry"), "the store is not last: {twice}");
    let skips = cpp(&src(&format!(
        "        if s[j] == 0:\n            continue\n{dp}"
    )));
    assert!(
        !skips.contains("carry"),
        "a round may skip the store: {skips}"
    );
    let first = cpp(&src(dp).replace("for j in i.after", "for j in I"));
    assert!(
        !first.contains("carry"),
        "j.prev! is unproven from I's first: {first}"
    );
}

#[test]
fn an_assert_builds_its_message_out_of_line() {
    let cpp = cpp(concat!(
        "fn cell(w: Int, c: Int) -> Int:\n",
        "    assert 0 <= c < w, \"column {c} is outside 0..{w}\"\n",
        "    assert c != 7, \"seven\"\n",
        "    assert c != 8, \"{c} is eight\"\n",
        "    assert c != 9, \"{c} is nine\"\n",
        "    c\n",
        "print(cell(read(), read()))\n",
    ));
    assert!(
        cpp.contains("if (__builtin_expect(!(0 <= c && c < w), 0)) { [=]() mutable __attribute__((noinline, cold)) { cpl::fail("),
        "{cpp}"
    );
    assert!(
        cpp.contains(
            "if (__builtin_expect(!(c != 7), 0)) { [=]() mutable __attribute__((noinline, cold))"
        ),
        "{cpp}"
    );
    assert!(!cpp.contains("[&]() __attribute__"), "{cpp}");
    assert!(
        cpp.contains("inline ll cell("),
        "the message is not counted against inlining: {cpp}"
    );
}

#[test]
fn a_range_of_a_sorted_bag_passes_its_constant_bounds_to_one_call() {
    let out = cpp(concat!(
        "var b = SortedBag<Int>()\n",
        "for x in [5, 2, 5, 1]:\n",
        "    let _ = b.add(x)\n",
        "let lo = 2\n",
        "print(b.range(2..).first ?? -1, b.range(..=5).last ?? -1, b.range(lo..6).len)\n",
    ));
    for call in [
        "_range_first(b, cpl::OptInt(2), true, cpl::OptInt(), false)",
        "_range_last(b, cpl::OptInt(), true, cpl::OptInt(5), true)",
    ] {
        assert!(out.contains(call), "no `{call}` in\n{out}");
    }
    // A bound that is not a constant is read once, into a local.
    assert!(out.contains("cpl::OptInt(6), false)"), "{out}");
    assert!(
        !out.contains("bool t"),
        "a constant bound bound to a local:\n{out}"
    );
    assert!(
        !out.contains("[&]() -> cpl::OptInt"),
        "a view's read in a lambda:\n{out}"
    );
}

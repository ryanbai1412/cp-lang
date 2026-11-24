//! The shapes each C++ counter counts, and the near misses it must not.

use crate::cpp_counts::*;

#[test]
fn a_part_returned_by_value_is_a_copy_and_by_reference_is_not() {
    let by_value = "std::vector<ll> f(const S& self) {\n    return self.xs;\n}\n";
    let lent = "const std::vector<ll>& f(const S& self) {\n    return self.xs;\n}\n";
    let local = "std::vector<ll> f() {\n    std::vector<ll> xs;\n    return xs;\n}\n";
    assert_eq!(container_copies(by_value), 1);
    assert_eq!(container_copies(lent), 0);
    assert_eq!(container_copies(local), 0);
    let element = "std::string f(const S& self, ll t) {\n    return self.agg[t];\n}\n";
    assert_eq!(container_copies(element), 1);
    let after_lambda = "ll f(ll u) {\n  auto t = [&]() -> std::vector<ll> {\n    return v;\n  }();\n  return need[u];\n}\n";
    assert_eq!(container_copies(after_lambda), 0);
    let forwarded = "inline const std::vector<ll>& A_view(const A& self) {\n  return self.xs;\n}\ninline std::vector<ll> B_view(const B& self) {\n  return A_view(self.a);\n}\n";
    assert_eq!(container_copies(forwarded), 1);
    let cached = "inline std::vector<ll> T_plain(T& self) {\n  return self.xs;\n}\n";
    assert_eq!(container_copies(cached), 1);
}

#[test]
fn a_place_passed_to_a_fn_that_moves_it_into_a_struct_is_a_sunk_copy() {
    let cpp = "inline H wrap(std::vector<ll> xs) {\n  return H{std::move(xs)};\n}\nvoid solve() {\n  s = (*wrap(a).xs).size() + wrap(f(a)).xs.size() + wrap(std::move(a)).xs.size();\n}\n";
    assert_eq!(sunk_copies(cpp), 1);
    assert_eq!(
        sunk_copies(&cpp.replace("std::vector<ll> xs", "const std::vector<ll>& xs")),
        0
    );
}

#[test]
fn a_header_states_a_count_once_and_above_zero() {
    assert_eq!(declared("// expect: ok\nprint(1)\n", "copies"), Ok(0));
    assert_eq!(
        declared("// expect: ok\n// copies: 2\nprint(1)\n", "copies"),
        Ok(2)
    );
    assert_eq!(declared("print(1)\n// copies: 2\n", "copies"), Ok(0));
    assert!(declared("// copies: 0\n", "copies").is_err());
    assert!(declared("// copies: two\n", "copies").is_err());
    assert!(declared("// copies: 1\n// copies: 1\n", "copies").is_err());
    assert_eq!(declared("// std_functions: 3\n", "std_functions"), Ok(3));
}

#[test]
fn a_shortened_std_function_counts_at_each_use() {
    let cpp = "template<class...T>using Qy=std::function<T...>;\n\
               struct M { Qy<ll(ll, ll)> add; Qy<ll(ll, ll)> mul; };\n\
               // Qy<ll()> in a comment\n\
               std::function<void()> f; XQy<int> not_it;\n";
    assert_eq!(std_functions(cpp), 3);
}

#[test]
fn a_place_is_a_dotted_name_or_an_index_through_cpl_at() {
    for place in ["xs", "self.rows", "_a.b_1", " xs ", "cpl::at(rows, i)"] {
        assert!(is_place(place), "{place}");
    }
    for not in [
        "",
        "1x",
        "a.",
        "a..b",
        "a-b",
        "f(x)",
        "cpl::at(rows, cpl::x)",
        "cpl::at(rows",
    ] {
        assert!(!is_place(not), "{not}");
    }
}

#[test]
fn a_part_is_a_field_an_element_or_a_cpl_at() {
    for part in ["self.xs", "cpl::at(rows, i)", "self.agg[t]"] {
        assert!(is_part(part), "{part}");
    }
    for not in ["xs", "xs[i]", "self.a[f(x)]", "cpl::at(a, i)[j]", "f(x)"] {
        assert!(!is_part(not), "{not}");
    }
}

#[test]
fn a_body_line_says_what_it_returns() {
    let cases = [
        (
            "std::vector<ll> f(const S& self) {",
            Some("std::vector<ll>"),
        ),
        ("inline std::vector<ll> f() {", Some("std::vector<ll>")),
        ("static ll g(ll x) {", Some("ll")),
        (
            "const std::vector<ll>& f(const S& self) {",
            Some("const std::vector<ll>&"),
        ),
        ("std::string* p(ll x) {", Some("std::string*")),
        (
            "const std::vector<ll>&f(const S& s) {",
            Some("const std::vector<ll>&"),
        ),
        ("ll*p(ll x) {", Some("ll*")),
        (
            "auto t = [&]() -> std::vector<ll> {",
            Some("std::vector<ll>"),
        ),
        ("else if (x) {", None),
        ("for (ll i = 0; i < n; i++) {", None),
        ("cpl::Bound<A, B>(x) {", None),
        ("const T& (x) {", None),
        ("f(x) {", None),
        ("std::vector<ll> f(x)", None),
    ];
    for (line, want) in cases {
        assert_eq!(returned_type(line), want, "{line}");
    }
}

#[test]
fn a_scalar_declaration_names_its_local() {
    let cases = [
        ("int f = 0;", Some("f")),
        ("for (ll i = 0; i < n; i++) {", Some("i")),
        ("ll my_x;", Some("my_x")),
        ("cpl::OptPos p = cpl::none;", Some("p")),
        ("std::vector<ll> f;", None),
        ("int = 0;", None),
        ("int f-1 = 0;", None),
        ("int", None),
    ];
    for (line, want) in cases {
        assert_eq!(scalar_decl(line), want, "{line}");
    }
}

#[test]
fn a_container_copied_into_an_optional_is_an_optional_copy() {
    assert_eq!(
        optional_copies("f(std::optional<std::vector<ll>>(a));\n"),
        1
    );
    assert_eq!(
        optional_copies(
            "  g(std::optional<std::vector<ll>>(self.xs), std::optional<std::string>(s));\nh(std::optional<std::set<ll>>(xs));\n"
        ),
        3
    );
    for not in [
        "std::optional<ll>(a);",
        "std::optional<std::vector<ll>>(f(a));",
        "std::optional<std::vector<ll>> x;",
        "std::optional<std::vector<ll>>{a};",
        "std::optional<std::vector<ll> (a)",
        "std::optional<std::vector<std::vector<ll>>(a)",
    ] {
        assert_eq!(optional_copies(not), 0, "{not}");
    }
    for ty in ["std::vector<ll>", "std::string", "std::deque<int>"] {
        assert!(is_container(ty), "{ty}");
    }
    for ty in ["ll", "cpl::Bag<ll>", "const std::vector<ll>&"] {
        assert!(!is_container(ty), "{ty}");
    }
}

#[test]
fn a_container_local_from_a_place_is_a_deep_copy() {
    let cases = [
        ("std::vector<ll> v = a;", 1),
        ("std::map<ll, ll> m = self.m;", 1),
        ("std::vector<std::pair<ll, ll>> v = cpl::at(rows, i);", 1),
        ("const std::vector<ll> v = a;", 1),
        ("std::vector<ll>& r = a;", 0),
        ("std::vector<ll>* p = a;", 0),
        ("const std::vector<ll>& r = a;", 0),
        ("std::vector<ll> v = f(a);", 0),
        ("std::vector<ll> v(a);", 0),
        ("std::vector<ll> = a;", 0),
        ("ll v = a;", 0),
        ("std::vector<ll> v = a;\nstd::string s = b;", 2),
    ];
    for (cpp, want) in cases {
        assert_eq!(container_copies(cpp), want, "{cpp}");
    }
}

#[test]
fn a_tuple_of_containers_from_a_pairs_fields_is_a_deep_copy() {
    let cases = [
        ("std::tuple<std::vector<ll>, ll> t(p.first, p.second);", 1),
        ("std::tuple<ll, ll> t(p.first, p.second);", 0),
        ("std::tuple<std::vector<ll>, ll> t(a, p.second);", 0),
        ("std::tuple<std::vector<ll>, ll> t(p.first, b);", 0),
        (
            "auto t = std::tuple<std::vector<ll>, ll>(p.first, p.second);",
            0,
        ),
    ];
    for (cpp, want) in cases {
        assert_eq!(container_copies(cpp), want, "{cpp}");
    }
}

#[test]
fn a_lambda_capturing_a_container_by_value_is_a_deep_copy() {
    let held = "std::vector<ll> xs;\nstd::vector<ll> ys;\n";
    let cases = [
        ("auto f = [xs = xs](ll y) {", 1),
        ("auto f = [xs = xs, ys = ys, n = n](ll y) {", 2),
        ("auto f = [&xs = xs, n = n](ll y) {", 0),
        ("auto f = [zs = zs](ll y) {", 0),
        ("auto f = [xs](ll y) {", 0),
        ("auto f = [xs = xs];", 0),
        ("int xs = 0;\nauto f = [xs = xs](ll y) {", 0),
    ];
    for (line, want) in cases {
        let cpp = format!("{held}{line}\n");
        assert_eq!(container_copies(&cpp), want, "{cpp}");
    }
    let unnamed = "std::vector<ll> & = a;\nauto f = [c = ](ll y) {\n";
    assert_eq!(container_copies(unnamed), 0);
}

#[test]
fn a_part_returned_counts_only_inside_a_fn_returning_a_container() {
    let scalar = "ll f(const S& self) {\n  return self.xs;\n}\n";
    assert_eq!(container_copies(scalar), 0);
    let closed = "std::vector<ll> f(const S& s) {\n  return s.xs;\n}\nreturn s.ys;\n";
    assert_eq!(container_copies(closed), 1);
    let not_lending =
        "ll& g(S& s) {\n  return s.n;\n}\nstd::vector<ll> f(S& s) {\n  return g(s);\n}\n";
    assert_eq!(container_copies(not_lending), 0);
    let two =
        "std::vector<ll> f(const S& s) {\n  if (c) {\n    return s.xs;\n  }\n  return s.ys;\n}\n";
    assert_eq!(container_copies(two), 2);
}

#[test]
fn only_a_std_function_alias_names_one() {
    let cases = [
        (
            "template<class...T>using Qy=std::function<T...>;",
            Some("Qy"),
        ),
        ("using Q_y = std::function<void()>;", Some("Q_y")),
        ("using Q y = std::function<void()>;", None),
        ("using = std::function<void()>;", None),
        ("using V = std::vector<ll>;", None),
        ("using namespace std;", None),
    ];
    for (line, want) in cases {
        assert_eq!(function_alias(line), want, "{line}");
    }
    assert_eq!(uses_of("Qy<ll()> a; Qy<int> b;", "Qy"), 2);
    for not in [
        "Qy a;",
        "XQy<int> a;",
        "a_Qy<int> a;",
        "ns::Qy<int> a;",
        "9Qy<int>",
    ] {
        assert_eq!(uses_of(not, "Qy"), 0, "{not}");
    }
    assert_eq!(uses_of("(Qy<int> a)", "Qy"), 1);
}

#[test]
fn a_map_entry_read_or_changed_is_a_copy() {
    assert_eq!(
        map_get_copies("std::optional<std::vector<ll>> t = cpl::map_get(m, k);\nstd::optional<ll> u = cpl::map_get(m, k);\nstd::optional<std::vector<ll>> v = w;\n"),
        1
    );
    assert_eq!(map_get_copies(""), 0);
    assert_eq!(
        map_get_copies("std::optional<std::vector<ll>> t = cpl::map_get(m, k);\nstd::optional<std::vector<int>> u = cpl::map_get(m, j);\n"),
        2
    );
    assert_eq!(entry_copies(""), 0);
    assert_eq!(entry_copies("f(std::vector<ll>(cpl::map_or(m, k, d)), std::vector<ll>(cpl::map_or(m, j, d)));\ng(std::vector<int>(cpl::map_or(n, k, d)));\n"), 2);
    assert_eq!(
        entry_copies(
            "auto a = std::vector<ll>(cpl::map_or(m, k, d));\nauto b = cpl::map_or(m, k, d);\n"
        ),
        1
    );
}

#[test]
fn a_block_that_sizes_a_domain_then_gives_a_list_is_a_copy() {
    let block = |sizes: &str, ret: &str| {
        format!("  auto xs = [&]() -> std::vector<ll> {{\n{sizes}    return {ret};\n  }}();\n")
    };
    assert_eq!(block_copies(&block("    D_len = n;\n", "xs")), 1);
    assert_eq!(
        block_copies(&block("    D_len = n;\n    E_len = m;\n", "xs_2")),
        1
    );
    assert_eq!(block_copies(&block("", "xs")), 0);
    assert_eq!(block_copies(&block("    D = n;\n", "xs")), 0);
    assert_eq!(block_copies(&block("    D_len = n;\n", "f(xs)")), 0);
    assert_eq!(block_copies(&block("    D_len = n;\n", "xs.v")), 0);
    let no_return = "auto xs = [&]() -> std::vector<ll> {\n  D_len = n;\n  xs;\n";
    assert_eq!(block_copies(no_return), 0);
    let ends = "auto xs = [&]() -> std::vector<ll> { D_len = n;\n  D_len = n;\n  return xs;\n";
    assert_eq!(block_copies(ends), 0);
    let at_end = "auto xs = [&]() -> std::vector<ll> {\n  D_len = n;\n";
    assert_eq!(block_copies(at_end), 0);
}

#[test]
fn items_split_at_depth_0() {
    assert_eq!(
        items("a, f(b, c), std::pair<x, y>, [d, e], {g, h}"),
        ["a", "f(b, c)", "std::pair<x, y>", "[d, e]", "{g, h}"]
    );
    assert_eq!(items(" a ,b"), ["a", "b"]);
    assert_eq!(items("a, "), ["a"]);
    assert!(items("").is_empty());
    assert!(items("  ").is_empty());
}

#[test]
fn a_sunk_copy_needs_a_by_value_heap_parameter_moved_into_the_result() {
    let fns = concat!(
        "inline H wrap(ll k, std::vector<ll> xs) {\n",
        "  return H{k, std::move(xs)};\n",
        "}\n",
        "inline H pack(Big b) {\n",
        "  return H{std::move(b)};\n",
        "}\n",
        "inline H keep(std::vector<ll> xs) {\n",
        "  return H{xs};\n",
        "}\n",
        "inline H tmpl(F0 xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
        "inline H unused([[maybe_unused]] std::vector<ll> xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
        "inline H later(std::vector<ll> xs) {\n",
        "  auto h = H{1};\n",
        "  h.v = std::move(xs);\n",
        "  return h;\n",
        "}\n",
        "H bare(std::vector<ll> xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
        "wrap_it(std::vector<ll> xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
        "inline H wrap_2(std::vector<ll> xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
        "inline H built(std::vector<ll> xs) {\n",
        "  auto h = H{std::move(xs)};\n",
        "  return h;\n",
        "}\n",
        "inline std::vector<ll> pass(std::vector<ll> xs) {\n",
        "  return std::move(xs);\n",
        "}\n",
        "ll scalar(ll xs) {\n",
        "  return H{std::move(xs)};\n",
        "}\n",
    );
    let calls = [
        ("wrap(1, a)", 1),
        ("wrap(a, 1)", 0),
        ("wrap(1, self.a) + wrap(2, cpl::at(rs, i))", 2),
        ("wrap(1, f(a))", 0),
        ("rewrap(1, a)", 0),
        ("x.wrap(1, a)", 1),
        ("wrap(1, g(a, b), c)", 0),
        ("wrap(1", 0),
        ("pack(b)", 1),
        ("keep(a) + tmpl(a) + unused(a) + later(a) + scalar(a)", 0),
        ("bare(a)", 1),
        ("wrap_it(a)", 0),
        ("wrap_2(a)", 1),
        ("built(a) + pass(a)", 0),
    ];
    for (call, want) in calls {
        let cpp = format!("{fns}void solve() {{\n  s = {call};\n}}\n");
        assert_eq!(sunk_copies(&cpp), want, "{call}");
    }
    let no_type = "inline wrap(xs) {\n  return H{std::move(xs)};\n}\n  s = wrap(a);\n";
    assert_eq!(sunk_copies(no_type), 0);
}

#[test]
fn a_fn_lent_its_value_that_returns_a_part_by_value_is_a_lent_part_copy() {
    let cases = [
        ("std::vector<ll> f(const S& self) {\n  return self.xs;\n", 1),
        (
            "inline std::vector<ll> f(ll k, const S& self) {\n  return self.xs[k];\n",
            1,
        ),
        (
            "std::vector<ll> f(const S& self, ll k) {\n  return self.xs;\n",
            1,
        ),
        ("std::vector<ll> f(S& self) {\n  return self.xs;\n", 0),
        (
            "std::vector<ll> f(const S& other) {\n  return self.xs;\n",
            0,
        ),
        ("std::vector<ll> f(const S& self) {\n  return self;\n", 0),
        ("std::vector<ll> f(const S& self) {\n  return self.xs\n", 0),
        ("std::vector<ll> f(const S& self) {\n  return .xs;\n", 0),
        ("std::vector<ll> f(const S& self) {\n  s = self.xs;\n", 0),
        ("ll f(const S& self) {\n  return self.n;\n", 0),
        ("std::vector<ll> f(const S& self)\n  return self.xs;\n", 0),
        (
            "std::vector<ll> f(const S& selfie) {\n  return self.xs;\n",
            0,
        ),
    ];
    for (cpp, want) in cases {
        assert_eq!(lent_part_copies(cpp), want, "{cpp}");
    }
}

#[test]
fn a_flat_struct_holds_no_container_or_std_function_even_through_another() {
    let flat = |cpp: &str| flat_structs(&cpp.lines().collect::<Vec<_>>());
    let set = |names: &[&str]| {
        names
            .iter()
            .map(|n| n.to_string())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let nested = "struct P {\n  ll a = 0;\n  Q q;\n};\nstruct Q {\n  int b = 0;\n};\n";
    assert_eq!(flat(nested), set(&["P", "Q"]));
    assert_eq!(
        flat(&nested.replace("int b = 0;", "std::vector<ll> b;")),
        set(&[])
    );
    assert_eq!(
        flat(&nested.replace("int b = 0;", "std::function<ll(ll)> b;")),
        set(&[])
    );
    assert_eq!(flat(&nested.replace("Q q;", "R q;")), set(&["Q"]));
    let methods = "struct M {\n  ll a = 0;\n  auto xs() { return std::vector<ll>{a}; }\n  bool operator==(const std::vector<ll>& o) const;\n};\n";
    assert_eq!(flat(methods), set(&["M"]));
    let after =
        "std::vector<ll> g;\nstruct A {\n  ll a = 0;\n};\nstruct B {\n  std::string s;\n};\n";
    assert_eq!(flat(after), set(&["A"]));
    assert_eq!(
        flat("struct A {\n  ll a = 0;\n};\nstd::vector<ll> g;\n"),
        set(&["A"])
    );
}

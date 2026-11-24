//! What the runtime around the program does: flushing, output bytes and
//! `Int` arithmetic C++ can't overflow.

use super::cpp;

#[test]
fn windows_writes_output_in_binary_mode() {
    // Else each `\n` is printed as `\r\n` there, unlike the interpreter.
    let out = cpp("print(1)\n");
    let block = &out[out.rfind("#ifdef _WIN32").expect("a Windows block")..];
    let block = &block[..block.find("#endif").expect("its #endif")];
    assert!(
        block.contains("_setmode(_fileno(stdout), _O_BINARY)"),
        "{out}"
    );
    assert!(
        out.contains("#ifdef _WIN32\n#include <fcntl.h>\n#include <io.h>\n#endif\n"),
        "{out}"
    );
}

#[test]
fn interactive_flushes_before_each_read_not_after_each_print() {
    let out = cpp("@!interactive\nfor _ in 0..3:\n    print(\"R\")\n    let x: Int = read()\n    print(x)\nprint(\"A\")\n");
    let solve = out
        .split("void solve()")
        .nth(1)
        .unwrap()
        .split("}  // namespace prog")
        .next()
        .unwrap();
    assert!(!solve.contains("flush"), "{solve}");
    assert!(
        out.contains("if (out.n) { flush(); fflush(stdout); }"),
        "{out}"
    );
}

#[test]
fn an_input_refill_reads_into_its_own_buffer_and_zeroes_nothing() {
    // Growing `in` before read(2) would zero 64 KiB per refill, which an
    // interactive program pays once per answer.
    let out = cpp("let wire = Wire()\nlet x: Int = wire.ask(\"? 1\")\nprint(\"! {x}\")\n");
    let refill = &out[out.find("inline bool more()").expect("the reader refills")..];
    let refill = &refill[..refill.find("\n}").unwrap_or(refill.len())];
    assert!(
        refill.contains("read(0, chunk,") && !refill.contains("resize"),
        "{refill}"
    );
}

#[test]
fn a_printed_number_and_its_separator_go_out_in_one_write() {
    let out =
        cpp("let n: 0..=100 = read()\nfor i in 0..n: print(i, i % 7)\nprint(n, \"x\", n + 1)\n");
    assert!(
        out.contains("cpl::put_int(i, ' '); cpl::put_int(i % 7, '\\n');"),
        "{out}"
    );
    assert!(out.contains("cpl::put_int(n, ' '); cpl::put_str("), "{out}");
    assert!(
        out.contains("cpl::put_char(' '); cpl::put_int(n + 1, '\\n');"),
        "{out}"
    );
    let body = &out[out.find("void solve()").expect("a solve fn")..];
    assert!(!body.contains("put_line"), "{body}");
}

#[test]
fn a_shift_not_proved_in_range_traps_in_a_checked_build() {
    let out = cpp("let k: Int = read()\nvar x = 3\nx <<= k\nprint(1 << k, x)\n");
    assert!(out.contains("cpl::ishl(1, "), "{out}");
    assert!(
        out.contains("x = cpl::ishl(x, cpl::shamt(k, 64));"),
        "{out}"
    );
    let proved = cpp("let k: 0..=62 = read()\nprint(1 << k)\n");
    assert!(
        proved.contains("1LL << k") || proved.contains("1 << k"),
        "{proved}"
    );
    assert!(!proved.contains("cpl::ishl("), "{proved}");
}

#[test]
fn an_int_op_not_proved_in_range_never_overflows_cpp() {
    let out = cpp("let n: Int = read()\nfor i in 0..n: print(n * 2, i + 1, n - i)\n");
    assert!(out.contains("cpl::imul(n, 2)"), "{out}");
    assert!(out.contains("cpl::isub(n, i)"), "{out}");
    assert!(out.contains("i + 1"), "{out}");
    assert!(
        out.contains("(ll)((unsigned long long)a * (unsigned long long)b)"),
        "{out}"
    );
}

#[test]
fn a_narrow_range_len_needs_no_overflow_check() {
    // Its span always fits in `ll`, so `len` in a hot loop is a subtraction (cf2066f).
    let out = cpp("domain P = read(1..=500)\nlet r = P.first!..=P.last!\nprint(r.len)\n");
    let len = &out[out.find("ll len() const").expect("Range::len")..];
    let check = len.find("range length overflow").expect("the wide check");
    assert!(
        len[..check].contains("if constexpr (sizeof(T) < sizeof(ll)) return (ll)span();"),
        "{out}"
    );
}

#[test]
fn every_runtime_piece_comes_after_the_pieces_it_needs() {
    let mut seen = std::collections::BTreeSet::new();
    for piece in crate::runtime::pieces() {
        for need in piece.needs {
            assert!(
                seen.contains(need),
                "{} needs {need}, which comes later or is missing",
                piece.name
            );
        }
        assert!(
            seen.insert(piece.name),
            "two pieces are named {}",
            piece.name
        );
    }
}

#[test]
fn a_zero_filled_table_comes_from_calloc() {
    // `vector(n, 0)` writes every page; calloc's pages stay unmapped until used.
    let out = cpp("var t: [80_000_000]0..=500 = 0\nt[7] = 3\nprint(t[7])\n");
    assert!(out.contains("cpl::zeros<uint16_t>(80000000)"), "{out}");
    assert!(out.contains("calloc(n, sizeof(T))"), "{out}");
    let grid = cpp("let n: 1..=5000 = read()\ndomain R = 0..n\nvar g: [R, R]Int = 0\ng[R(0), R(0)] = n\nprint(g[R(0), R(0)])\n");
    assert!(
        grid.contains("cpl::zero_grid<ll, 2>({R_len, R_len})"),
        "{grid}"
    );
    let small = cpp("var t: [1000]0..=500 = 0\nt[7] = 3\nprint(t[7])\n");
    assert!(!small.contains("zeros<"), "{small}");
    let ones = cpp("let n: 1..=9_000_000 = read()\nvar t: [n]Int = 1\nt[0] = n\nprint(t[0])\n");
    assert!(!ones.contains("zeros<"), "{ones}");
    let opts =
        cpp("let n: 1..=9_000_000 = read()\nvar t: [n]?Int = None\nt[0] = n\nprint(t[0] ?? 0)\n");
    assert!(!opts.contains("zeros<"), "{opts}");
}

#[test]
fn a_sort_by_holds_half_a_buffer_and_no_sorted_copy() {
    // Keys, a u32 order and n/2 buffer cells; the permutation is applied in place.
    let out = cpp(
        "fn f(x: Int) -> Int: (x * 37) % 11\nvar xs = [5, 3, 9, 1]\nxs.sort(by: f)\nprint(xs)\n",
    );
    assert!(out.contains("cpl::sort_by("), "{out}");
    assert!(out.contains("std::vector<V> buf(b, b + n / 2)"), "{out}");
    assert!(out.contains("std::vector<uint32_t> order"), "{out}");
    assert!(!out.contains("std::vector<T> sorted"), "{out}");
}

#[test]
fn a_sort_by_a_cell_compares_in_place() {
    // A key read from a domain-indexed table needs no cached keys or order;
    // one that ends in the element itself sorts unstably, as no order shows.
    let src = |key: &str| {
        format!(
            "let n: 1..=9 = read()\ndomain I = 0..n\nlet a: [I]Int = read()\n\
             var ord = I.collect()\nord.sort(by: i => {key})\nprint(ord[0].ord)\n"
        )
    };
    let unique = cpp(&src("(a[i], i)"));
    assert!(unique.contains("std::sort(ord"), "{unique}");
    assert!(!unique.contains("cpl::sort_by("), "{unique}");
    let ties = cpp(&src("a[i]"));
    assert!(ties.contains("cpl::stable_sort_lt(ord"), "{ties}");
    assert!(!ties.contains("cpl::sort_by("), "{ties}");
}

#[test]
fn a_plain_sort_is_unstable_only_where_equal_means_the_same() {
    // Equal tuples of integers are one value, so no order of them shows;
    // a Float (-0.0 and 0.0) or an `impl Ord` can tie values that differ.
    let src = |decl: &str, elem: &str| {
        format!(
            "{decl}var xs: List<{elem}> = []\nxs.sort()\nprint(xs.len)\nprint(sorted(xs).len)\n"
        )
    };
    let same = cpp(&src("", "(Int, Bool, Char, Str, (a: Int, b: Int))"));
    assert!(same.contains("cpl::sort_seq<true>(xs.begin()"), "{same}");
    assert!(same.contains("cpl::sorted<true>(xs)"), "{same}");
    let float = cpp(&src("", "(Int, F64)"));
    let own = cpp(&src(
        "struct P(a: Int, b: Int)\nimpl Ord for P:\n    fn lt(a, b) = a.a < b.a\n",
        "P",
    ));
    for kept in [&float, &own] {
        assert!(kept.contains("cpl::sort_seq(xs.begin()"), "{kept}");
        assert!(kept.contains("cpl::sorted(xs)"), "{kept}");
    }
    let source = src("", "(Int, Int)");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source.as_str());
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let (profiled, _) = crate::emit_profiled(&program, "t.cp", &crate::Target::default(), "t.prof");
    assert!(
        profiled.contains("cpl::sort_seq(xs.begin()"),
        "a profile counts the interpreter's comparisons: {profiled}"
    );
}

#[test]
fn a_long_token_is_scanned_a_word_at_a_time_and_copied_out_by_chunk() {
    let cpp = cpp("let s: Str = read()\nprint(s.len)\n");
    let read = &cpp[cpp.find("inline void read_str_into(").unwrap()..];
    let read = &read[..read.find("\n}").unwrap()];
    assert!(
        read.contains("need_value()") && !read.contains("need_input()"),
        "{read}"
    );
    assert!(read.contains("0x8080808080808080ULL"), "{read}");
}

#[test]
fn a_structs_own_order_spells_every_comparison_through_its_lt() {
    let cpp = cpp(concat!(
        "struct P(a: Int, b: Int)\nimpl Ord for P:\n    fn lt(a, b) = a.a < b.a\n",
        "let n: Int = read()\nlet p = P(n, 0)\nlet q = P(0, n)\n",
        "if p < q: print(1)\nif p > q: print(2)\nif p <= q: print(3)\nif p >= q: print(4)\n",
    ));
    for test in [
        "(P_lt(p, q))",
        "(P_lt(q, p))",
        "(!P_lt(q, p))",
        "(!P_lt(p, q))",
    ] {
        assert!(cpp.contains(&format!("if {test} {{")), "{test}: {cpp}");
    }
}

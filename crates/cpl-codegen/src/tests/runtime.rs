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
fn interactive_flushes_after_each_print() {
    let said = "cpl::put_line(); cpl::flush(); fflush(stdout);";
    assert!(cpp("@!interactive\nprint(1)\n").contains(said));
    assert!(!cpp("print(1)\n").contains(said));
}

#[test]
fn a_shift_not_proved_in_range_traps_in_a_checked_build() {
    let out = cpp("let k: 0..=63 = read()\nvar x = 3\nx <<= k\nprint(1 << k, x)\n");
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

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

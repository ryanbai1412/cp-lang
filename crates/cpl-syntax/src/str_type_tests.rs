//! `Str(len: …)` and `Str of "…"`: string types that bound the length and
//! the characters (R02.3.2).

use crate::*;

fn codes(src: &str) -> Vec<String> {
    parse_file(src)
        .diagnostics
        .into_iter()
        .map(|d| d.code.0.to_string())
        .collect()
}

#[test]
fn str_types_take_a_length_range_and_an_alphabet() {
    for ok in [
        "let s: Str(len: 1..=n) = read()\n",
        "let s = read<Str(len: n..=n) of \"01\">()\n",
        "let s = read<Str of \"UDLR\">()\n",
    ] {
        assert_eq!(codes(ok), Vec::<String>::new(), "{ok}");
    }
    assert!(!codes("let s: Str(len: Int) = read()\n").is_empty());
    assert!(!codes("let s: Str(1..=3) = read()\n").is_empty());
}

//! A list passed where an array is wanted lends its cells: the array shares
//! the list's storage until one of them writes, so each pass costs nothing
//! per element (dmopc20c7p2's `edges.equal_range` on a 1.6e6-edge list).

mod common;
mod counting;
use common::program;

const SOURCE: &str = "let m = read<Int>()
var xs = [(a: i, b: i % 7) for i in 0..1000]
var t = 0
for k in 0..m:
    t += xs.equal_range(k % 1000, by: e => e.a).len
    if k == 0: xs[0].b = 9
print(t, xs[0].b)";

/// What the program printed and the bytes it allocated.
fn run(prog: &cpl_ir::Program, m: usize) -> (String, usize) {
    let mut out = Vec::with_capacity(64);
    let input = format!("{m}\n");
    let (observed, usage) = counting::measure(|here| {
        here.observe(
            prog,
            cpl_interp::Io::fixed(input.as_bytes()),
            &mut out,
            None,
        )
    });
    assert!(observed.trap.is_none(), "the program runs");
    (String::from_utf8(out).unwrap(), usage.allocated)
}

#[test]
fn a_list_passed_as_an_array_is_not_copied_per_pass() {
    const M: usize = 200;
    let prog = program(SOURCE);
    run(&prog, M);
    let (out, few) = run(&prog, M);
    let (_, many) = run(&prog, 4 * M);
    assert_eq!(out, format!("{M} 9\n"));
    // A copy of the 1000-element list is at least 16 bytes an element.
    let per_pass = many.saturating_sub(few) / (3 * M);
    assert!(
        per_pass < 1000 * 8,
        "{per_pass} bytes allocated per pass: {few} at m = {M}, {many} at m = {}",
        4 * M
    );
}

#[test]
fn an_array_lent_by_a_list_keeps_the_values_it_was_passed() {
    let prog = program(
        "var xs = [3, 1, 2]
fn total<D>(a: [D]Int) -> Int = a.sum()
let before = total(xs)
xs[0] = 10
print(before, total(xs), xs)",
    );
    let (out, _) = run(&prog, 0);
    assert_eq!(out, "6 13 10 1 2\n");
}

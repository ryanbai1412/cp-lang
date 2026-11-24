//! A view over a named array (`runs`, `windows`, `chunks`, `split`) borrows
//! its piece's cells: the bytes a pass allocates grow with the pieces, never
//! with the array once per piece (noip07p1's `for run in xs.runs()`).

mod common;
mod counting;
use common::program;

/// The bytes a run of `prog` allocates on an `n`-cell array of alternating
/// 0 and 1, and what it printed.
fn run(prog: &cpl_ir::Program, n: usize) -> (String, usize) {
    let cells: Vec<&str> = (0..n).map(|i| ["0", "1"][i % 2]).collect();
    let input = format!("{n}\n{}\n", cells.join(" "));
    let mut out = Vec::with_capacity(64);
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
fn a_view_over_a_named_array_allocates_per_piece_not_per_cell() {
    const N: usize = 1000;
    let mut copied = Vec::new();
    for binding in ["let", "var"] {
        for (view, cells_of) in [
            ("runs()", N),
            ("windows(1)", N),
            ("chunks(1)", N),
            ("split(0)", N / 2),
        ] {
            let source = format!(
                "let n: 1..=100000 = read()
{binding} xs: [0..n]0..=1 = read()
var k = 0
for w in xs.{view}: k += w.len
print(k)"
            );
            let prog = program(&source);
            let (out, few) = run(&prog, N);
            let (_, many) = run(&prog, 4 * N);
            assert_eq!(out, format!("{cells_of}\n"), "{binding} xs.{view}");
            // A piece's own values cost a few hundred bytes; a copy of the
            // array per piece costs at least 8 bytes for each of its n cells.
            let per_piece = many.saturating_sub(few) / (3 * N);
            if per_piece >= N {
                copied.push(format!(
                    "{binding} xs.{view}: {few} bytes at n = {N}, {many} at n = {}",
                    4 * N
                ));
            }
        }
    }
    assert!(
        copied.is_empty(),
        "views copied their array:\n{}",
        copied.join("\n")
    );
}

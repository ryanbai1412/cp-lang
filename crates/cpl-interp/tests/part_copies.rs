//! A statement that reads or writes one part of a big container copies at
//! most that part: counted on the VM, the bytes handed out over `4m` such
//! statements are within one container's copy of those over `m`.

mod common;
mod counting;
use common::program;

/// Each case holds a 65536-cell container and touches it `m` times.
const CASES: &[(&str, &str)] = &[
    (
        "a global struct's list field, `op=` at an index",
        "type Money = 0..=1e10
struct B:
    t: List<Money>
var b = B([0] * 65536)
let m = read<Int>()
for i in 0..m: b.t[i % 1000] max= i
print(b.t[5])",
    ),
    (
        "a global struct's list field, read and stored at an index",
        "struct B:
    t: List<Int>
var b = B([0] * 65536)
let m = read<Int>()
for i in 0..m: b.t[i % 1000] = b.t[i % 1000] + i
print(b.t[5])",
    ),
];

fn bytes(prog: &cpl_ir::Program, m: usize) -> usize {
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
    usage.allocated
}

#[test]
fn touching_one_part_of_a_container_copies_none_of_the_rest() {
    const M: usize = 200;
    const ONE_COPY: usize = 65536 * 8;
    let mut grew = Vec::new();
    for (place, source) in CASES {
        let prog = program(source);
        let few = bytes(&prog, M);
        let many = bytes(&prog, 4 * M);
        if many > few + ONE_COPY {
            grew.push(format!(
                "{place}: {few} bytes at m = {M}, {many} at m = {}",
                4 * M
            ));
        }
    }
    assert!(
        grew.is_empty(),
        "each statement copied the container:\n{}",
        grew.join("\n")
    );
}

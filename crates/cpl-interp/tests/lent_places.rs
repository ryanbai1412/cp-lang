//! A `mut` call on a struct reached through a field (`w.t.f()`) lends the
//! struct itself: the callee's writes land in it without copying its lists.
//! Counted on the VM, each call must add no allocation.

mod common;
mod counting;
use common::program;

const NESTED: &[(&str, &str)] = &[
    ("a field of a global", "w.t.bump(i % 1000)"),
    ("a field of self", "w.go(i % 1000)"),
    ("a field of a mut arg", "go(mut w, i % 1000)"),
];

fn source(call: &str) -> String {
    format!(
        "struct Slot:
    var key: Int
    var cnt: Int
struct T:
    var slots: List<Slot>
    var free: List<Int>
    fn bump(mut self, u: Int) -> Int:
        slots[u].cnt += 1
        1
struct W:
    var t: T
    fn go(mut self, u: Int) -> Int = self.t.bump(u)
fn go(mut w: W, u: Int) -> Int = w.t.bump(u)
let m = read<Int>()
var w = W(T([Slot(i, 1) for i in 0..1000], []))
var z = 0
for i in 0..m: z += {call}
print(z + w.t.slots[0].cnt)"
    )
}

fn allocations(prog: &cpl_ir::Program, m: usize) -> usize {
    let mut out = Vec::with_capacity(64);
    let input = format!("{m}\n");
    let before = counting::usage().1;
    cpl_interp::run(prog, input.as_bytes(), &mut out).expect("the program runs");
    counting::usage().1 - before
}

#[test]
fn a_mut_call_on_a_nested_field_copies_nothing() {
    const M: usize = 200;
    let mut grew = Vec::new();
    for (place, call) in NESTED {
        let prog = program(&source(call));
        allocations(&prog, M);
        let few = allocations(&prog, M);
        let many = allocations(&prog, 4 * M);
        if many > few + 8 {
            grew.push(format!(
                "{place}: {few} allocations at m = {M}, {many} at m = {}",
                4 * M
            ));
        }
    }
    assert!(
        grew.is_empty(),
        "each call copied the struct's lists:\n{}",
        grew.join("\n")
    );
}

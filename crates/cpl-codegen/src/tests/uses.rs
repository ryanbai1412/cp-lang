//! The Uses table: what may happen to each heap binding.

use super::lower;
use crate::uses::{Home, KEPT};

fn uses_of(source: &str, name: &str) -> crate::uses::Use {
    let target = crate::Target::default();
    let program = crate::prepared(&lower(source), &target);
    let slot = program
        .main_locals
        .iter()
        .position(|l| l.name == name)
        .expect("the local is in main");
    crate::emit::Emitter::new(&program, &target)
        .uses
        .of_binding(Home::Top, slot)
}

#[test]
fn a_list_read_only_by_index_is_neither_mutated_nor_moved() {
    let source = "let n = read<Int>()
for v in 0..n:
    let xs = [v, v + 1]
    print(xs[0] + xs.len)
";
    let xs = uses_of(source, "xs");
    assert!(!xs.mutated && !xs.moved && xs.escapes.is_none(), "{xs:?}");
}

#[test]
fn a_pushed_list_is_mutated_and_one_read_whole_may_move() {
    let source = "let n = read<Int>()
for v in 0..n:
    var xs = [v]
    xs.push(v)
    let ys = xs
    print(ys.len)
";
    let xs = uses_of(source, "xs");
    assert!(xs.mutated && xs.moved, "{xs:?}");
}

#[test]
fn a_list_a_kept_closure_captures_escapes() {
    let source = "let n = read<Int>()
for v in 0..n:
    let xs = [v]
    let fs = [() => xs.len]
    print(fs[0]())
";
    assert_eq!(uses_of(source, "xs").escapes, Some(KEPT));
}

#[test]
fn a_read_of_a_value_set_while_it_lives_is_a_copy() {
    let out = super::cpp(
        "var best = (gap: 9, id: 0)
let a = best min= (gap: 4, id: 7)
let b = best min= (gap: 4, id: 8)
print(best.gap, best.id, Int(a), Int(b))
",
    );
    assert!(
        !out.contains("std::tuple<ll, ll>& t"),
        "`min=` compares with what `best` was, so that is copied, not lent:\n{out}"
    );
}

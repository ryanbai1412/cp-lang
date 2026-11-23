//! Fn values the C++ calls directly instead of through `std::function`.

use super::cpp;

#[test]
fn a_fn_field_read_into_a_local_calls_its_only_closure() {
    let out = cpp("use std.tree.{Rooted}
use std.algebra.{Monoid}
domain V = 0..5
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let t = Rooted(adj, root: 0)
print(t.reroot(Monoid(0, (a, b) => a + b), (c, _, _) => c + 1, (s, _) => s))
");
    assert!(
        !out.contains("f = mon.op"),
        "every `Monoid<Int>` holds the same `op`, so `f(..)` calls it directly:\n{out}"
    );
}

#[test]
fn a_fn_parameter_the_body_never_names_is_not_a_std_function() {
    let out = cpp("use std.heap.{Heap}
var h = Heap<Int>()
h.push(3)
h.push(1)
print(h.pop()!)
");
    assert!(
        !out.contains("std::function"),
        "`Heap.new` without `by:` never reads it, so it is a `bool` placeholder:\n{out}"
    );
}

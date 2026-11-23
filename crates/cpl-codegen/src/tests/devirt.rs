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

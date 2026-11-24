// expect: ok, output "4\n3"
// rule: 08.2.11
// A static fn's struct generics bind from an untyped lambda's body as a free generic fn's do: `V` in `Lift.of(next: V -> ?V)` is what `up[v]` gives.
use std.tree.Lift
domain V = 0..5
let up: [V]?V = [V] v => if v.idx > 0: Some(V(v.idx - 1)) else: None
print(Lift.of(v => up[v]).depth(V(4)))
struct Walk<V>(next: V -> ?V):
    pub fn of(next: V -> ?V) -> Walk<V> = Walk<V>(next)
    pub fn steps(self, v: V) -> Int:
        var n = 0
        var at = v
        while let p = next(at):
            n += 1
            at = p
        n
print(Walk.of(v => up[v]).steps(V(3)))

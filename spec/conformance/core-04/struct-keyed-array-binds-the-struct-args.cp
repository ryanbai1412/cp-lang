// expect: ok, output "6 6 3"
// rule: 04.5
// A generic `[Pair<W>]T` given a `[Pair<V>]Int` binds `W` to `V` through the key struct's
// arguments, as a free fn, a method-form fn and a generic element alike.
use std.pair.{Pair}
domain V = 0..4
fn f<W>(a: [Pair<W>]Int) -> Int = a.len
fn g<W, T>(self: [Pair<W>]T) -> Int = W.count * 0 + self.len
fn h<W, T>(self: [Pair<W>]T, at: Pair<W>) -> T = self[at]
let w: [Pair<V>]Int = [Pair<V>] p => p.lo.ord + p.hi.ord
print(f(w), w.g(), w.h(Pair<V>.at(V(1), V(2))!))

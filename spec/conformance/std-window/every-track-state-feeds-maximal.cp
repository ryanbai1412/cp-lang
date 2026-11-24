// expect: ok, output "1 2 3 1 1 2 3 2"
// rule: std.seq.maximal
// A `Track` from another module is a `state:` for `maximal` like a `track`: a `SortedBag` and a `Tracked` (sequence-patterns §5).
let a = [5, 7, 6, 9]
let spread = a.maximal(state: SortedBag<Int>(), ok: b => b.last! - b.first! <= 2)
let sum = a.maximal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: t => t.state <= 20)
print([w.len for w in spread], [w.len for w in sum])

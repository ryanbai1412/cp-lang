// expect: ok, output "2 b 3 | 1 1"
// rule: 04b.1.2
// A slice of a `[0..n]T` is a view keyed by the array's positions, like a slice over a named
// domain, so it is the `Slice<S>` a generic fn such as `runs` returns.
let t: [0..4]Char = ['a', 'b', 'b', 'c']
let w = t[1..3]
print(w.len, w.nth(0), w.keys.end, "|", t.runs().map(r => r.len).collect()[0], t[3..].len)

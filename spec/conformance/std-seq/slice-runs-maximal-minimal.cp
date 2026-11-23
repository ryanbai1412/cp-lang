// expect: ok, output "2 3\n1 2 1\n2 1 1\n0 2 1\n2 2 1\n0 2 3"
// rule: 04.4.1
use std.seq.{maximal, minimal}
print([r.len for r in [1, 2, 4, 5, 6].runs(link: (x, y) => y == x + 1)])
print([1, 1, 2].maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 1).map(w => w.len))
print([1, 1, 2].maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 1, anchor: Anchor.Start).map(w => w.len))
print([1, 2, 3].minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3).map(w => w.map(.len) ?? 0))
print([1, 2, 3].minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3, anchor: Anchor.Start).map(w => w.map(.len) ?? 0))
print([2, 1, 0].minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3).map(w => w.map(.len) ?? 0))

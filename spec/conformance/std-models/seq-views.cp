// expect: ok, output "ok"
// input: "1 300\n"
// model: filter
// model: map
// model: rev
// model: Rev
// model: items
// model: collect
// model: drop
// model: drop_last
// model: take
// model: take_last
// model: Slice
// model: nth
// model: rebase
// model: step
// model: even
// model: odd
// model: Stepped
// model: StepKeys
// model: keyed
// model: keys
// model: rekey
// The std views (filter, map, rev, slices, steps, keyed, rekey) over a List against index
// loops: each op draws a short random list and cut points and checks every view's elements,
// keys and length. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let len = rng.int(0, 9)
    let a = [rng.int(-5, 5) for _ in 0..len]
    let c = rng.int(0, 11)
    let st = rng.int(1, 4)
    let lo = rng.int(0, len)
    let hi = rng.int(lo, len)
    let i = rng.int(0, 10)
    let m = if c < len: c else: len
    let s = a[lo..hi]
    var why = ""
    if a.filter(x => x > 0).collect() != [x for x in a if x > 0]: why = "filter"
    elif a.filter(x => x > 0).items.collect() != [(j, a[j]) for j in 0..len if a[j] > 0]: why = "filter items"
    elif a.map(x => x * 3).collect() != [x * 3 for x in a]: why = "map"
    elif i < len and a.map(x => x * 3)[i] != a[i] * 3: why = "map index"
    elif a.rev.collect() != [a[len - 1 - j] for j in 0..len] or a.rev.src != a: why = "rev"
    elif a.rev.items.collect() != [(len - 1 - j, a[len - 1 - j]) for j in 0..len]: why = "rev items"
    elif i < len and a.rev.nth(i) != a[len - 1 - i]: why = "rev nth"
    elif List(a.rev.take(c)) != [a[len - 1 - j] for j in 0..m]: why = "rev take"
    elif List(a.rev.drop(c)) != [a[len - 1 - j] for j in m..len]: why = "rev drop"
    elif List(a.rev.take_last(c)) != [a[m - 1 - j] for j in 0..m]: why = "rev take_last"
    elif List(a.rev.drop_last(c)) != [a[len - 1 - j] for j in 0..len - m]: why = "rev drop_last"
    elif a.items.collect() != [(j, a[j]) for j in 0..len]: why = "items"
    elif a.take(c).collect() != [a[j] for j in 0..m] or (m > 0 and a.take(c).keys != 0..m): why = "take"
    elif a.drop(c).collect() != [a[j] for j in m..len] or (m < len and a.drop(c).keys != m..len): why = "drop"
    elif a.take_last(c).collect() != [a[j] for j in len - m..len]: why = "take_last"
    elif a.drop_last(c).collect() != [a[j] for j in 0..len - m]: why = "drop_last"
    elif s.len != hi - lo or s.is_empty != (hi == lo) or (lo < hi and s.keys != lo..hi): why = "slice shape"
    elif s.items.collect() != [(j, a[j]) for j in lo..hi] or s.collect() != [a[j] for j in lo..hi]: why = "slice items"
    elif i < len and a.nth(i) != a[i]: why = "nth"
    elif i < hi - lo and (s.nth(i) != a[lo + i] or s.rebase.nth(i) != a[lo + i]): why = "slice nth/rebase"
    elif a.step(st).len != (len + st - 1) / st or a.step(st).is_empty != (len == 0): why = "step len"
    elif a.step(st).each().collect() != [a[j] for j in 0..len if j % st == 0]: why = "step each"
    elif i < a.step(st).len and a.step(st).nth(i) != a[i * st]: why = "step nth"
    elif i < len and i % st == 0 and a.step(st).at(i) != a[i]: why = "step at"
    elif a.step(st).keys.len != a.step(st).len: why = "step keys len"
    elif any(a.step(st).keys.contains(j) != (0 <= j < len and j % st == 0) for j in -2..12): why = "step keys"
    elif any((j in s.step(st).keys) != (lo <= j < hi and (j - lo) % st == 0) for j in -2..12): why = "slice step keys"
    elif a.even.each().collect() != [a[j] for j in 0..len if j % 2 == 0]: why = "even"
    elif a.odd.each().collect() != [a[j] for j in 0..len if j % 2 == 1] or a.odd.len != len / 2: why = "odd"
    elif a.keyed().items.collect() != [(j, a[j]) for j in 0..len]: why = "keyed"
    elif a.keyed(keys: 10..).items.collect() != [(10 + j, a[j]) for j in 0..len]: why = "keyed keys"
    elif a.keyed(by: x => x * 2).items.collect() != [(x * 2, x) for x in a]: why = "keyed by"
    elif a.keyed().collect() != a: why = "keyed walk"
    elif a.keys != 0..len: why = "keys"
    elif i < len and a.rekey(7..7 + len)[7 + i] != a[i]: why = "rekey"
    elif a.rekey(7..7 + len).len != len: why = "rekey len"
    if why != "":
        bad = "op {k}: {why} on {a} (c {c}, step {st}, {lo}..{hi}, i {i})"
        break
print(if bad == "": "ok" else: bad)

// expect: ok, output "ok"
// input: "1 2000\n"
// model: LazyForest
// model: ForestSeq
// A `LazyForest`'s sequences against plain lists: seeded random push, split_at, append, apply
// and reverse over a growing set of sequences, with len and get checked after each op and
// to_list at the end. Reads `seed n`; prints the first op that differs.
use std.lazy.{ForestSeq, LazyForest}
let seed0 = read<Int>()
let steps = read<Int>()
var f = LazyForest<Int, Int>(apply: (v, k, g) => v + g, compose: (g, h) => g + h)
var ids: List<ForestSeq> = []
var want: List<List<Int>> = []
for _ in 0..4:
    ids.push(f.seq())
    want.push([])
var x = seed0
var bad = ""
for step in 0..steps:
    if bad != "": break
    x = (x * 1103515245 + 12345) % 2147483648
    let i = x % ids.len
    let n = want[i].len
    let op = (x / 7) % 5
    if op == 0 or n < 2:
        f.push(ids[i], step)
        want[i].push(step)
    elif op == 1:
        let k = (x / 11) % (n + 1)
        let u = f.split_at(ids[i], k)
        want.push(want[i][k..].to_list())
        want[i].truncate(k)
        ids.push(u)
    elif op == 2:
        let j = (x / 13) % ids.len
        if j != i:
            f.append(ids[i], ids[j])
            want[i].extend(want[j])
            want[j] = []
    elif op == 3:
        let (lo, hi) = (x / 17 % n, x / 19 % n)
        let r = min(lo, hi)..max(lo, hi)
        f.apply(ids[i], r, 3)
        for p in r: want[i][p] += 3
    else:
        let (lo, hi) = (x / 23 % n, x / 29 % n)
        f.reverse(ids[i], min(lo, hi)..max(lo, hi))
        want[i][min(lo, hi)..max(lo, hi)].reverse()
    let m = want[i].len
    if f.len(ids[i]) != m or (m > 0 and f.get(ids[i], m / 2) != want[i][m / 2]):
        bad = "op {step}: sequence {i} differs"
for (k, s) in ids.items:
    if bad == "" and f.to_list(s) != want[k]: bad = "end: sequence {k} differs"
print(if bad == "": "ok" else: bad)

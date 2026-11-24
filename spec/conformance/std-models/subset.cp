// expect: ok, output "ok"
// input: "1 300\n"
// model: Subset
// Subset<B> against a List<Bool>: seeded random with, without, set, shifted, complement and the
// four set operators, with members, len, first, items, each, subsets and supersets compared after
// each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
domain B = 0..6

fn members(a: List<Bool>) -> List<B> = [B(i) for i in 0..6 if a[i]]

fn subset_of(a: List<Bool>, b: List<Bool>) -> Bool:
    for i in 0..6:
        if a[i] and not b[i]: return false
    true

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var s = Subset<B>.empty
var a = List.filled(6, false)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let i = rng.int(0, 5)
    var t = Subset<B>.empty
    var c = List.filled(6, false)
    for j in 0..6:
        if rng.int(0, 1) == 1:
            t = t.with(B(j))
            c[j] = true
    if op == 0:
        s = s.with(B(i))
        a[i] = true
    elif op == 1:
        s = s.without(B(i))
        a[i] = false
    elif op == 2:
        let on = rng.int(0, 1) == 1
        s.set(B(i), on)
        a[i] = on
    elif op == 3:
        let d = rng.int(-7, 7)
        s = s.shifted(d)
        a = [j - d >= 0 and j - d < 6 and a[j - d] for j in 0..6]
    elif op == 4:
        s = s.complement()
        a = [not x for x in a]
    elif op == 5:
        s = s | t
        a = [a[j] or c[j] for j in 0..6]
    elif op == 6:
        s = s & t
        a = [a[j] and c[j] for j in 0..6]
    elif op == 7:
        s = s ^ t
        a = [a[j] != c[j] for j in 0..6]
    elif op == 8:
        s = s - t
        a = [a[j] and not c[j] for j in 0..6]
    else:
        s = Subset<B>.single(B(i))
        a = [j == i for j in 0..6]
    let want = members(a)
    if s.items.collect() != want or [b for b in s] != want:
        bad = "op {k}: items {s.items.collect()} want {want}"
    elif s.len != want.len or s.is_empty != (want.len == 0):
        bad = "op {k}: len"
    elif s.first() != want.first:
        bad = "op {k}: first"
    elif (B(i) in s) != a[i]:
        bad = "op {k}: contains {i}"
    elif (s == Subset<B>.full) != (want.len == 6) or Subset<B>.full.len != 6:
        bad = "op {k}: full"
    else:
        var subs = 0
        for g in s.subsets:
            if not subset_of([B(j) in g for j in 0..6], a): bad = "op {k}: subsets yields {g}"
            subs += 1
        var sups = 0
        for g in s.supersets:
            if not subset_of(a, [B(j) in g for j in 0..6]): bad = "op {k}: supersets yields {g}"
            sups += 1
        if bad == "" and (subs != 1 << want.len or sups != 1 << (6 - want.len)):
            bad = "op {k}: {subs} subsets, {sups} supersets"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

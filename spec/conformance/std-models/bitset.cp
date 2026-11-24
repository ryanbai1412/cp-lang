// expect: ok, output "ok"
// input: "1 300\n"
// model: Bitset
// Bitset<D> over a domain spanning three words against a List<Bool>: seeded random point edits,
// set algebra with a random other set (built by `of`, `from`, `single`, `full`, `empty`), shifts
// both ways, range sets and flips and subset sums, with every query compared after each op.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
domain D = 0..130
let m = 130

fn bits(b: Bitset<D>) -> List<Bool> = [b.contains(D(i)) for i in 0..m]

fn shift(a: List<Bool>, k: Int) -> List<Bool> = [i - k >= 0 and i - k < m and a[i - k] for i in 0..m]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var b = Bitset<D>()
var a = List.filled(m, false)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 22)
    let i = rng.int(0, m - 1)
    let kind = rng.int(0, 4)
    var o = List.filled(m, false)
    let dense = rng.int(0, 3)
    for j in 0..m:
        o[j] = rng.int(0, 3) < dense
    let ob = if kind == 0: Bitset<D>.of([D(j) for j in 0..m if o[j]])
        elif kind == 1: Bitset<D>.from([D] v => o[Int(v)])
        elif kind == 2: Bitset<D>.single(D(i))
        elif kind == 3: Bitset<D>.full()
        else: Bitset<D>.empty()
    if kind == 2: o = [j == i for j in 0..m]
    elif kind == 3: o = List.filled(m, true)
    elif kind == 4: o = List.filled(m, false)
    let sh = rng.int(-m - 5, m + 5)
    let lo = rng.int(0, m)
    let hi = rng.int(lo, m)
    let r = D.cuts.nth(lo)..D.cuts.nth(hi)
    if op == 0:
        if b.add(D(i)) != (not a[i]): bad = "op {k}: add {i}"
        a[i] = true
    elif op == 1:
        if b.remove(D(i)) != a[i]: bad = "op {k}: remove {i}"
        a[i] = false
    elif op == 2:
        b.toggle(D(i))
        a[i] = not a[i]
    elif op == 3:
        b = if kind % 2 == 0: b.with(D(i)) else: b.without(D(i))
        a[i] = kind % 2 == 0
    elif op == 4:
        b = b.set(D(i), kind < 2)
        a[i] = kind < 2
    elif op == 5:
        b = b.complement()
        a = [not x for x in a]
    elif op == 6 and kind == 0 and dense == 0:
        b.clear()
        a = List.filled(m, false)
    elif op == 7:
        b.union(ob)
        a = [a[j] or o[j] for j in 0..m]
    elif op == 8:
        b.intersect(ob)
        a = [a[j] and o[j] for j in 0..m]
    elif op == 9:
        b.subtract(ob)
        a = [a[j] and not o[j] for j in 0..m]
    elif op == 10:
        b.symmetric(ob)
        a = [a[j] != o[j] for j in 0..m]
    elif op == 11:
        b = if kind < 2: b | ob else: b.bitor(ob)
        a = [a[j] or o[j] for j in 0..m]
    elif op == 12:
        b = if kind < 2: b & ob else: b.bitand(ob)
        a = [a[j] and o[j] for j in 0..m]
    elif op == 13:
        b = if kind < 2: b ^ ob else: b.bitxor(ob)
        a = [a[j] != o[j] for j in 0..m]
    elif op == 14:
        if b.meets(ob) != any(a[j] and o[j] for j in 0..m): bad = "op {k}: meets"
        b = b.shifted(sh)
        a = shift(a, sh)
    elif op == 15:
        let s = if sh < 0: -sh else: sh
        if kind < 2:
            b.shift_up(s)
            a = shift(a, s)
        else:
            b.shift_down(s)
            a = shift(a, -s)
    elif op == 16:
        let s = if sh < 0: -sh else: sh
        b.shift_or(s)
        let t = shift(a, s)
        a = [a[j] or t[j] for j in 0..m]
    elif op == 17:
        b.union_shifted(ob, sh)
        let t = shift(o, sh)
        a = [a[j] or t[j] for j in 0..m]
    elif op == 18:
        let on = kind < 3
        if kind == 0: b.set_range(r) else: b.set_range(r, on:)
        for j in lo..hi: a[j] = on
    elif op == 19:
        b.flip_range(r)
        for j in lo..hi: a[j] = not a[j]
    elif op == 20:
        let ws = [rng.int(0, 40) for _ in 0..rng.int(0, 5)]
        b = Bitset<D>.subset_sums(ws)
        a = List.filled(m, false)
        a[0] = true
        for w in ws:
            let t = shift(a, w)
            a = [a[j] or t[j] for j in 0..m]
    let ones = [j for j in 0..m if a[j]]
    let pivot = rng.int(0, m - 1)
    let nk = rng.int(0, ones.len + 1)
    let want_next = [j for j in ones if j >= i]
    let want_prev = [j for j in ones if j <= i]
    if bad != "": break
    if bits(b) != a: bad = "op {k}: bits"
    elif b.len != ones.len or b.count != ones.len or b.is_empty != (ones.len == 0): bad = "op {k}: len"
    elif b.each().map(d => Int(d)).collect() != ones: bad = "op {k}: each"
    elif b.each_rev().map(d => Int(d)).collect() != [ones[ones.len - 1 - j] for j in 0..ones.len]: bad = "op {k}: each_rev"
    elif b.first() != (if ones.len == 0: None else: Some(D(ones[0]))): bad = "op {k}: first"
    elif b.find(d => Int(d) >= pivot) != [D(j) for j in ones if j >= pivot].first: bad = "op {k}: find"
    elif b.rfind(d => Int(d) <= pivot) != [D(j) for j in ones if j <= pivot].last: bad = "op {k}: rfind"
    elif b.any(d => Int(d) % 7 == pivot % 7) != any(j % 7 == pivot % 7 for j in ones): bad = "op {k}: any"
    elif b.next(D(i)) != (if want_next.len == 0: None else: Some(D(want_next[0]))): bad = "op {k}: next {i}"
    elif b.prev(D(i)) != (if want_prev.len == 0: None else: Some(D(want_prev[want_prev.len - 1]))): bad = "op {k}: prev {i}"
    elif b.count_below(D(i)) != [j for j in ones if j < i].len: bad = "op {k}: count_below {i}"
    elif b.count_range(r) != [j for j in ones if j >= lo and j < hi].len: bad = "op {k}: count_range {lo}..{hi}"
    elif b.nth(nk) != (if nk < ones.len: Some(D(ones[nk])) else: None): bad = "op {k}: nth {nk}"
    if bad != "": break
print(if bad == "": "ok" else: bad)

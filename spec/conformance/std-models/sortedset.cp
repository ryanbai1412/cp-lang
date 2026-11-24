// expect: ok, output "ok"
// input: "1 300\n"
// model: SortedSet
// SortedSet<Int> against a List<Bool> over keys 0..=16: seeded random add, remove, set,
// pop_first, pop_last, drain, split_at with join, absorb, clear, take and reserve, with len,
// first, last, each, each_rev, to_list, contains, rank, nth, first_true, last_true and a range
// view compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bag.{SortedSet, SortedRange}

fn keys(s: List<Bool>) -> List<Int> = [i for i in 0..s.len if s[i]]

fn opt(a: List<Int>, i: Int) -> ?Int = if 0 <= i and i < a.len: Some(a[i]) else: None

fn last_of(a: List<Int>) -> ?Int = opt(a, a.len - 1)

fn rev(a: List<Int>) -> List<Int> = [a[a.len - 1 - i] for i in 0..a.len]

fn has(s: List<Bool>, x: Int) -> Bool = 0 <= x and x < s.len and s[x]

fn info(v: SortedRange<Int>) -> (?Int, ?Int, Int, List<Int>, List<Int>) =
    (v.first, v.last, v.len, v.each().collect(), v.rev.collect())

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var want = List.filled(17, false)
var s: SortedSet<Int> = SortedSet()
var bad = ""
for k in 0..n:
    let op = rng.int(0, 11)
    let x = rng.int(0, 16)
    if op <= 2:
        if s.add(x) == want[x]: bad = "op {k}: add {x}"
        want[x] = true
    elif op == 3:
        if s.remove(x) != want[x]: bad = "op {k}: remove {x}"
        want[x] = false
    elif op == 4:
        let on = rng.int(0, 1) == 1
        s.set(x, on)
        want[x] = on
    elif op == 5:
        let a = keys(want)
        if rng.int(0, 1) == 0:
            if s.pop_first() != opt(a, 0): bad = "op {k}: pop_first"
            if a.len > 0: want[a[0]] = false
        else:
            if s.pop_last() != last_of(a): bad = "op {k}: pop_last"
            if a.len > 0: want[a[a.len - 1]] = false
    elif op == 6:
        let t = rng.int(-1, 17)
        let side = rng.int(0, 2)
        let a = keys(want)
        var w: List<Int> = []
        var got: List<Int> = []
        if side == 0:
            got = s.drain(after: x, while: y => y < t)
            for y in a:
                if y > x:
                    if y >= t: break
                    w.push(y)
        elif side == 1:
            got = s.drain(before: x, while: y => y > t)
            for y in rev(a):
                if y < x:
                    if y <= t: break
                    w.push(y)
        else:
            got = s.drain(while: y => y < t)
            for y in a:
                if y >= t: break
                w.push(y)
        if got != w: bad = "op {k}: drain side {side} {x} {t} gave {got} want {w}"
        for y in w: want[y] = false
    elif op == 7:
        let hi = s.split_at(x)
        let a = keys(want)
        if s.to_list() != [y for y in a if y < x] or hi.to_list() != [y for y in a if y >= x]:
            bad = "op {k}: split_at {x} gave {s.to_list()} and {hi.to_list()}"
        s.join(hi)
    elif op == 8:
        let more = [rng.int(0, 16) for _ in 0..rng.int(0, 6)]
        var o = SortedSet.of(more)
        s.absorb(mut o)
        if not o.is_empty: bad = "op {k}: absorb left other non-empty"
        for y in more: want[y] = true
    else:
        let w = rng.int(0, 3)
        if w == 0:
            s.clear()
            want = List.filled(17, false)
        elif w == 1:
            let t = s.take()
            if not s.is_empty or s.len != 0 or s.first != None: bad = "op {k}: take left keys"
            s = t
        elif w == 2:
            s = SortedSet.of(keys(want) + keys(want))
        else:
            s.reserve(rng.int(0, 40))
    if bad != "": break
    let a = keys(want)
    let q = rng.int(-1, 17)
    let hi = rng.int(-1, 17)
    let i = rng.int(-1, a.len + 1)
    let ge = [y for y in a if y >= q]
    let le = [y for y in a if y <= q]
    let r = [y for y in a if q <= y and y < hi]
    let r2 = [y for y in a if y <= hi]
    if s.len != a.len or s.is_empty != (a.len == 0):
        bad = "op {k}: len {s.len} want {a.len}"
    elif s.to_list() != a or s.each().collect() != a or s.each_rev().collect() != rev(a):
        bad = "op {k}: keys {s.to_list()} want {a}"
    elif s.first != opt(a, 0) or s.last != last_of(a):
        bad = "op {k}: first or last"
    elif s.contains(q) != has(want, q) or (q in s) != has(want, q):
        bad = "op {k}: contains {q}"
    elif s.rank(q) != [y for y in a if y < q].len or s.nth(i) != opt(a, i):
        bad = "op {k}: rank {q} or nth {i}"
    elif s.first_true(y => y >= q) != opt(ge, 0) or s.last_true(y => y <= q) != last_of(le):
        bad = "op {k}: first_true or last_true {q}"
    elif info(s.range(q..hi)) != (opt(r, 0), last_of(r), r.len, r, rev(r)):
        bad = "op {k}: range {q}..{hi}"
    elif info(s.range(..=hi)) != (opt(r2, 0), last_of(r2), r2.len, r2, rev(r2)):
        bad = "op {k}: range ..={hi}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

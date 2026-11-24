// expect: ok, output "ok"
// input: "1 300\n"
// model: SortedBag
// model: SortedRange
// SortedBag<Int> against a sorted List<Int>: seeded random add, add_many, remove, remove_all,
// remove_at, pop_first, pop_last, take_first, drain_range, drain, split_at with join, absorb,
// clear, take and reserve on small keys so copies pile up, with every query (len, distinct,
// first, last, each, entries, rank, nth, first_true, last_true, sum, the cursors and a
// SortedRange view of every range form) compared after each op. Keys stay >= 0 because `..b`
// on a Range<Int> starts at 0. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bag.{SortedBag, SortedRange, BagCursor}

fn opt(a: List<Int>, i: Int) -> ?Int = if 0 <= i and i < a.len: Some(a[i]) else: None

fn last_of(a: List<Int>) -> ?Int = opt(a, a.len - 1)

fn cnt(a: List<Int>, x: Int) -> Int = [y for y in a if y == x].len

fn rev(a: List<Int>) -> List<Int> = [a[a.len - 1 - i] for i in 0..a.len]

fn drop1(a: List<Int>, x: Int) -> List<Int>:
    var out: List<Int> = []
    var done = false
    for y in a:
        if y == x and not done:
            done = true
        else:
            out.push(y)
    out

fn groups(a: List<Int>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    for y in a:
        if out.len > 0 and out[out.len - 1].0 == y:
            out[out.len - 1] = (y, out[out.len - 1].1 + 1)
        else:
            out.push((y, 1))
    out

fn inr(y: Int, f: Int, lo: Int, hi: Int) -> Bool:
    if f == 0: lo <= y and y < hi
    elif f == 1: lo <= y and y <= hi
    elif f == 2: y < hi
    elif f == 3: y <= hi
    elif f == 4: lo <= y
    else: true

fn key(b: SortedBag<Int>, c: ?BagCursor) -> ?Int = if let cc = c: Some(b.key_of(cc)) else: None

fn info(v: SortedRange<Int>) -> (?Int, ?Int, Int, Bool, List<Int>, List<Int>) =
    (v.first, v.last, v.len, v.is_empty, v.each().collect(), v.rev.collect())

fn view(b: SortedBag<Int>, f: Int, lo: Int, hi: Int) -> (?Int, ?Int, Int, Bool, List<Int>, List<Int>):
    if f == 0: info(b.range(lo..hi))
    elif f == 1: info(b.range(lo..=hi))
    elif f == 2: info(b.range(..hi))
    elif f == 3: info(b.range(..=hi))
    elif f == 4: info(b.range(lo..))
    else: info(b.range(0..))

fn drain_form(mut b: SortedBag<Int>, f: Int, lo: Int, hi: Int) -> List<Int>:
    if f == 0: b.drain_range(lo..hi).collect()
    elif f == 1: b.drain_range(lo..=hi).collect()
    elif f == 2: b.drain_range(..hi).collect()
    elif f == 3: b.drain_range(..=hi).collect()
    elif f == 4: b.drain_range(lo..).collect()
    else: b.drain_range(0..).collect()

fn draw(mut rng: Rng) -> List<Int> = [rng.int(0, 16) for _ in 0..rng.int(0, 6)]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var a = draw(mut rng).sorted()
var b = SortedBag.of(a)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 13)
    let x = rng.int(0, 16)
    if op <= 2:
        if not b.add(x): bad = "op {k}: add {x} returned false"
        a.push(x)
        a = a.sorted()
    elif op == 3:
        let c = rng.int(0, 3)
        b.add_many(x, c)
        for _ in 0..c: a.push(x)
        a = a.sorted()
    elif op == 4:
        if b.remove(x) != (cnt(a, x) > 0): bad = "op {k}: remove {x}"
        a = drop1(a, x)
    elif op == 5:
        let c = b.remove_all(x)
        if c != cnt(a, x): bad = "op {k}: remove_all {x} gave {c}"
        a = [y for y in a if y != x]
    elif op == 6:
        let want = [y for y in a if y >= x]
        if let c = b.cursor_ge(x):
            let y = b.key_of(c)
            if Some(y) != opt(want, 0): bad = "op {k}: cursor_ge {x} at {y}"
            b.remove_at(c)
            a = drop1(a, y)
        elif want.len > 0:
            bad = "op {k}: cursor_ge {x} missing"
    elif op == 7:
        if rng.int(0, 1) == 0:
            let got = b.pop_first()
            if got != opt(a, 0): bad = "op {k}: pop_first {got ?? -1}"
            if a.len > 0: a = drop1(a, a[0])
        else:
            let got = b.pop_last()
            if got != last_of(a): bad = "op {k}: pop_last {got ?? -1}"
            if a.len > 0: a = drop1(a, a[a.len - 1])
    elif op == 8:
        let got = b.take_first(y => y >= x)
        let want = opt([y for y in a if y >= x], 0)
        if got != want: bad = "op {k}: take_first >= {x} gave {got ?? -1}"
        if let w = want: a = drop1(a, w)
    elif op == 9:
        let f = rng.int(0, 5)
        let hi = rng.int(-1, 17)
        let got = drain_form(mut b, f, x, hi)
        let want = [y for y in a if inr(y, f, x, hi)]
        if got != want: bad = "op {k}: drain_range form {f} {x} {hi} gave {got}"
        a = [y for y in a if not inr(y, f, x, hi)]
    elif op == 10:
        let t = rng.int(-1, 17)
        let side = rng.int(0, 2)
        var want: List<Int> = []
        if side == 0:
            let got = b.drain(after: x, while: y => y < t)
            for y in a:
                if y > x:
                    if y >= t: break
                    want.push(y)
            if got != want: bad = "op {k}: drain after {x} while < {t} gave {got}"
        elif side == 1:
            let got = b.drain(before: x, while: y => y > t)
            for y in rev(a):
                if y < x:
                    if y <= t: break
                    want.push(y)
            if got != want: bad = "op {k}: drain before {x} while > {t} gave {got}"
        else:
            let got = b.drain(while: y => y < t)
            for y in a:
                if y >= t: break
                want.push(y)
            if got != want: bad = "op {k}: drain while < {t} gave {got}"
        for y in want: a = drop1(a, y)
    elif op == 11:
        let hi = b.split_at(x)
        if b.to_list() != [y for y in a if y < x] or hi.to_list() != [y for y in a if y >= x]:
            bad = "op {k}: split_at {x} gave {b.to_list()} and {hi.to_list()}"
        b.join(hi)
    elif op == 12:
        let more = draw(mut rng)
        var o = SortedBag.of(more)
        b.absorb(mut o)
        if not o.is_empty: bad = "op {k}: absorb left other non-empty"
        for y in more: a.push(y)
        a = a.sorted()
    else:
        let w = rng.int(0, 3)
        if w == 0:
            b.clear()
            a = []
        elif w == 1:
            let t = b.take()
            if not b.is_empty or b.len != 0 or b.first != None: bad = "op {k}: take left elements"
            b = t
        else:
            b.reserve(rng.int(0, 40))
    if bad != "": break
    let g = groups(a)
    let q = rng.int(-1, 17)
    let f = rng.int(0, 5)
    let hi = rng.int(-1, 17)
    let r = [y for y in a if inr(y, f, q, hi)]
    var fwd: List<(Int, Int)> = []
    var c = b.cursor_first()
    while let cc = c:
        fwd.push((b.key_of(cc), b.count_of(cc)))
        c = b.next_cursor(cc)
    var bwd: List<(Int, Int)> = []
    c = b.cursor_last()
    while let cc = c:
        bwd.push((b.key_of(cc), b.count_of(cc)))
        c = b.prev_cursor(cc)
    let ge = [y for y in a if y >= q]
    let le = [y for y in a if y <= q]
    let i = rng.int(-1, a.len + 1)
    if b.len != a.len or b.distinct != g.len or b.is_empty != (a.len == 0):
        bad = "op {k}: len {b.len} distinct {b.distinct} want {a.len} {g.len}"
    elif b.to_list() != a or b.each().collect() != a or b.each_rev().collect() != rev(a):
        bad = "op {k}: elements {b.to_list()} want {a}"
    elif b.first != opt(a, 0) or b.last != last_of(a):
        bad = "op {k}: first or last"
    elif b.entries().map(e => (e.key, e.count)).collect() != g or b.entries_rev().map(e => (e.key, e.count)).collect().len != g.len:
        bad = "op {k}: entries"
    elif fwd != g or bwd.len != g.len or [bwd[bwd.len - 1 - j] for j in 0..bwd.len] != g:
        bad = "op {k}: cursor walk {fwd} {bwd}"
    elif b.sum() != a.sum():
        bad = "op {k}: sum {b.sum()}"
    elif b.count(q) != cnt(a, q) or b.contains(q) != (cnt(a, q) > 0) or (q in b) != (cnt(a, q) > 0):
        bad = "op {k}: count {q}"
    elif b.rank(q) != [y for y in a if y < q].len or b.nth(i) != opt(a, i):
        bad = "op {k}: rank {q} or nth {i}"
    elif b.first_true(y => y >= q) != opt(ge, 0) or b.last_true(y => y <= q) != last_of(le):
        bad = "op {k}: first_true or last_true {q}"
    elif key(b, b.cursor_at(q)) != (if cnt(a, q) > 0: Some(q) else: None) or key(b, b.cursor_ge(q)) != opt(ge, 0) or key(b, b.cursor_le(q)) != last_of(le):
        bad = "op {k}: cursor_at, cursor_ge or cursor_le {q}"
    elif key(b, b.cursor_gt(q)) != opt([y for y in a if y > q], 0) or key(b, b.cursor_lt(q)) != last_of([y for y in a if y < q]):
        bad = "op {k}: cursor_gt or cursor_lt {q}"
    elif view(b, f, q, hi) != (opt(r, 0), last_of(r), r.len, r.len == 0, r, rev(r)):
        bad = "op {k}: range form {f} {q} {hi}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

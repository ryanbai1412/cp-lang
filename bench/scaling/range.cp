let n: Int = read()
var s = 0
for i in 0..8 * n:
    let r = i..i + 9
    if r.l.r != None: s += 1 // timed: range.l
    if r.r.l != None: s += 1 // timed: range.r
    if let (a, b) = r.halve(): s += a.len - b.len // timed: range.halve
    if r.mid.r != None: s += 1 // timed: range.mid
    s += r.clip(3..i + 5).len // timed: range.clip
    s += r.ends.0 // timed: range.ends
for (a, b) in (0..8 * n).splits: // timed: range.splits
    s += a.len - b.len
for i in 0..4 * n:
    let ge = (x: Int) => x >= i % n // within: range.first_true
    let le = (x: Int) => x <= i % n // within: range.last_true
    let gb = (x: Int) => x >= i % n // within: range.bsearch
    s += (0..n).first_true(ge) ?? 0 // timed: range.first_true
    s += (0..n).last_true(le) ?? 0 // timed: range.last_true
    if (0..n).bsearch(gb).r != None: s += 1 // timed: range.bsearch
let a = [0 for _ in 0..n]
for i in 0..8 * n:
    let g = a.ring.span(i % n, 5)
    s += g.len // timed: range.RingRange.len
    s += g.start // timed: range.RingRange.start
    s += g.end // timed: range.RingRange.end
    if g.contains((i + 2) % n): s += 1 // timed: range.RingRange.contains
for k in 0..32:
    for j in a.ring.span(k, n).keys: // timed: range.RingRange.keys
        s += j // timed: range.RingRange.keys
print(s)

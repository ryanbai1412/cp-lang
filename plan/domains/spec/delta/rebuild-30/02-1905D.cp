// cf1905d Cyclic MEX: the prefix mexes of a shift are non-decreasing. Moving the
// first element x to the back drops the first mex, caps every later mex at x, and
// appends n. Keep the mexes as a monotone deque of (value, count) blocks with
// their running total.
@!multitest
let n: 1..=1e6 = read()
let p = read<[n]0..n>()

var seen: [0..=n]Bool = false
var mex = 0
var q = Deque<(v: Int, c: Int)>()
var total = 0
for x in p:
    seen[x] = true
    while seen[mex]: mex += 1
    if let b = q.last, b.v == mex: q.pop(); q.push((v: mex, c: b.c + 1))
    else: q.push((v: mex, c: 1))
    total += mex

var best = total
for x in p:
    let f = q.pop_front()!
    total -= f.v
    if f.c > 1: q.push_front((v: f.v, c: f.c - 1))
    var cnt = 0
    while let b = q.last, b.v > x:
        q.pop()
        total -= b.v * b.c
        cnt += b.c
    q.push((v: x, c: cnt))
    total += x * cnt
    q.push((v: n, c: 1))
    total += n
    best max= total
print(best)

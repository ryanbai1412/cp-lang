// `x ?? break` and `x ?? continue` diverge just like `x ?? return`,
// both in `let` bindings and reassignments (cf2262b's `from = stop.right ?? break`).
let xs: List<?Int> = [some(10), none, some(20), some(30)]
var sum_skip = 0
for o in xs:
    let v = o ?? continue
    sum_skip += v

var sum_stop = 0
var cur: ?Int = some(0)
var steps = 0
while steps < 10:
    steps += 1
    let idx = cur ?? break
    let v = xs[idx] ?? break
    sum_stop += v
    cur = if idx + 1 < xs.len: some(idx + 1) else: none

var x = 0
for o in xs:
    x = o ?? break
print(sum_skip, sum_stop, x)

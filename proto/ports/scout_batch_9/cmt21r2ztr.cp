// original: corpus/submissions/4016530.cpp
// checker: proto/ports/scout_batch_9/harness/cmt21r2ztr_check.cpp
// cmt21r2ztr: n hidden numbers and m facts, each saying that the median
// of three of them is x. Print numbers that fit every fact, or NO.
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain Slot = 1..=n
let facts: [m](i: Slot, j: Slot, k: Slot, x: 1..=1e9) = read()

// A fact says: of its three numbers at most one is below x and at most one
// is above. So the only questions are "is slot s below x?" and "is it
// above x?", one pair for every slot and value some fact puts together.
const SPAN = 1073741824          // 2^30, above every value
fn key(s: Slot, x: Int) -> Int: s.label * SPAN + x
var keys: List<Int> = []
for f in facts:
    for s in [f.i, f.j, f.k]: keys.push(key(s, f.x))
let (Mark, mark) = compress(keys)
fn at(s: Slot, x: Int) -> Mark: mark.first_ge(key(s, x))!
fn slot(q: Mark) -> Slot: Slot(mark[q] / SPAN)
fn value(q: Mark) -> Int: mark[q] % SPAN

// Four statements for each mark: below, not below, above, not above. An
// arc u -> v says "if u holds then v holds".
domain Claim = 0..4 * Mark.len
fn below(q: Mark) -> Claim: Claim(4 * q.idx)
fn above(q: Mark) -> Claim: Claim(4 * q.idx + 2)
fn no(c: Claim) -> Claim: Claim(c.idx ^ 1)
var arcs: List<(u: Claim, v: Claim)> = []
fn either(a: Claim, b: Claim):
    arcs.push((no(a), b))
    arcs.push((no(b), a))

for f in facts:
    let (p, q, r) = (at(f.i, f.x), at(f.j, f.x), at(f.k, f.x))
    for (a, b) in [(p, q), (p, r), (q, r)]:
        either(no(below(a)), no(below(b)))
        either(no(above(a)), no(above(b)))
for q in Mark:
    either(no(below(q)), no(above(q)))
    // marks of one slot are neighbours, in order of value: a number is
    // above this value or below the next one, and never both, because a
    // number strictly between two values can be moved down onto one
    if let r = q + 1 and slot(q) == slot(r):
        either(above(q), below(r))
        either(no(above(q)), no(below(r)))

// Claims that force each other hold or fail together. If a claim forces its
// own denial round a cycle there is no answer. Otherwise believe whichever
// of the two comes later, since nothing later can force something earlier.
let (Knot, of) = scc(Graph.directed(arcs))
fn holds(c: Claim) -> Bool: of[c] > of[no(c)]
if any(of[below(q)] == of[no(below(q))] or of[above(q)] == of[no(above(q))] for q in Mark):
    print("NO")
else:
    print("YES")
    var number: [Slot]Int = 1
    for q in Mark:              // ascending, so the last word on a slot is its largest bound
        if not holds(below(q)): number[slot(q)] = value(q)
        if holds(above(q)): number[slot(q)] = value(q) + 1
    print(number)

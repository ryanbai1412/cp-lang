// Source: AtCoder AGC 077 A, Reverse A…B, https://atcoder.jp/contests/agc077/tasks/agc077_a
// limits: 2000 ms, 1024 MB
// Problem: strings X, Y over {A, B} of length N <= 5e5 (multitest, sum N <= 5e5). A move picks
//   l < r with X_l = A, X_r = B and reverses X[l..=r]. Turn X into Y in at most floor(N/2)
//   moves: print No, or Yes and the moves.
// Solution: moves keep the B count, so counts must match. Strip the common prefix and suffix
//   to the differing span l..=r; one big move reverses all of it (so X_l = A, X_r = B), with
//   other moves before it on X or after it on Y (undone, last first). The big move mirrors the
//   middle, so reconcile X's middle with Y's reversed middle: past equal heads, binary-search
//   the shortest prefix whose B counts agree again (A…B on one side, B…A on the other), reverse
//   it on the side where it reads left to right, recurse into both parts. O(N log N).

@!multitest
let n: 2..=500_000 = read()
let xs: Str = read()
let ys: Str = read()

if xs.count(c => c == 'B') != ys.count(c => c == 'B'): return print("No")
if xs == ys: return print("Yes\n0")
let l = common_prefix_len(xs, ys)
let r = n - 1 - common_suffix_len(xs, ys)
if xs[l] != 'A' or xs[r] != 'B': return print("No")

// Each string is one side of the big reversal: reversals on X happen before
// it, reversals on Y after it (so they are undone, last first).
enum Side:
    X
    Y
let text = [Side] s => if s == X: xs else: ys
let count_b = [Side] s => prefix([Int(c == 'B') for c in text[s]])

// Positions at, at + d, ..., end of a string.
struct Run(at: Int, end: Int, d: Int):
    @get fn len(self) -> Int = (end - at) * d + 1
    @get fn span(self) -> Range<Int> = min(at, end)..=max(at, end)
    @get fn rev(self) -> Run = Run(end, at, -d)
    fn take(self, k: Int) -> Run = Run(at, at + d * (k - 1), d)
    fn drop(self, k: Int) -> Run = Run(at + d * k, end, d)

// One stretch of the middle: per side, where its current letters are read
// in the input (`seen`) and where it sits in the real string on that side of
// the big reversal (`real`).
struct Block(seen: [Side]Run, real: [Side]Run):
    @get fn len(self) -> Int = seen[X].len
    fn each(self, f: Run -> Run) -> Block = Block(seen.map(f), real.map(f))
    fn head(self, s: Side) -> Char = text[s][seen[s].at]
    fn bs(self, s: Side, k: Int) -> Int = count_b[s].fold(seen[s].take(k).span)

var ops: [Side]List<Run> = []

fn reconcile(var b: Block):
    while b.len > 0 and b.head(X) == b.head(Y): b = b.each(v => v.drop(1))
    if b.len == 0: return
    let lead = if b.head(X) == 'B': 1 else: -1
    let k = (1..b.len).last_true(k => (b.bs(X, k) - b.bs(Y, k)) * lead > 0)! + 1
    reconcile(b.each(v => v.drop(k)))
    var blk = b.each(v => v.take(k))
    if blk.head(X) == 'B': blk = blk.each(v => v.rev)
    // The prefix is A…B on one side and B…A on the other: reverse it on the
    // side where its real position reads left to right.
    let side = if blk.real[X].d == 1: X else: Y
    ops[side].push(blk.real[side])
    blk.seen[side] = blk.seen[side].rev
    reconcile(blk)

let middle = Run(l + 1, r - 1, 1)
reconcile(Block(seen: [middle, middle.rev], real: [middle, middle.rev]))
let plan = ops[X] + [Run(l, r, 1)] + ops[Y].rev
print("Yes")
print(plan.len)
for op in plan: print(op.span.start + 1, op.span.end)

// AtCoder AGC 077 A: Reverse A…B.
//
// A reversal keeps the B count, so the counts must match. Let l..=r span the
// first and last differences. Every plan here spends one reversal on all of
// l..=r (the big one) with X_l = A and X_r = B, so those must hold. Other
// reversals go before it, on X, or after it, where we undo them from Y. The
// big one mirrors the middle, so a stretch that reads left to right on one
// side reads right to left on the other, and a reversal is legal on exactly
// the side where its A…B reads left to right.
//
// Reconcile the middle block by block. Strip the ends that already agree. At
// the first difference, the running B-count gap of x over y is ±1; take a
// prefix where it returns to 0 (binary search finds a crossing, since the gap
// is nonzero after one letter and 0 over the whole block). That prefix is
// A…B in one string and B…A in the other, so one reversal on its legal side
// makes both ends agree; recurse into it, then into the rest. Each reversal
// fixes two positions, so at most n / 2 of them.
use std.strings.{common_prefix_len, common_suffix_len}

@!multitest
let n: 2..=500_000 = read()
let xs: Str = read()
let ys: Str = read()

if xs.count('B') != ys.count('B'): return print("No")
if xs == ys: return print("Yes\n0")
let l = common_prefix_len(xs, ys)
let r = n - 1 - common_suffix_len(xs, ys)
if xs[l] != 'A' or xs[r] != 'B': return print("No")

let bs_x = prefix([Int(c == 'B') for c in xs])
let bs_y = prefix([Int(c == 'B') for c in ys])

// Positions at, at + d, ..., end of a string.
struct Run(at: Int, end: Int, d: Int):
    @get fn len(self) -> Int = (end - at) * d + 1
    @get fn span(self) -> Range<Int> = min(at, end)..=max(at, end)
    @get fn rev(self) -> Run = Run(end, at, -d)
    fn take(self, k: Int) -> Run = Run(at, at + d * (k - 1), d)
    fn drop(self, k: Int) -> Run = Run(at + d * k, end, d)

// One stretch of the middle seen four ways: x and y read its current
// contents out of the input strings; before and after place it in the real
// string before and after the big reversal.
struct Block(x: Run, y: Run, before: Run, after: Run):
    fn each(self, f: Run -> Run) -> Block = Block(f(x), f(y), f(before), f(after))

var before_big: List<Run> = []
var after_big: List<Run> = []

fn reconcile(var b: Block):
    while b.x.len > 0 and xs[b.x.at] == ys[b.y.at]: b = b.each(v => v.drop(1))
    if b.x.len == 0: return
    let lead = if xs[b.x.at] == 'B': 1 else: -1
    fn gap(k: Int) -> Int = bs_x.sum(b.x.take(k).span) - bs_y.sum(b.y.take(k).span)
    let k = bsearch_last(1..b.x.len, k => gap(k) * lead > 0)! + 1
    reconcile(b.each(v => v.drop(k)))
    var blk = b.each(v => v.take(k))
    if xs[blk.x.at] == 'B': blk = blk.each(v => v.rev)
    if blk.before.d == 1:
        before_big.push(blk.before)
        blk.x = blk.x.rev
    else:
        after_big.push(blk.after)
        blk.y = blk.y.rev
    reconcile(blk)

let middle = Run(l + 1, r - 1, 1)
reconcile(Block(middle, middle.rev, middle, middle.rev))
let ops = before_big + [Run(l, r, 1)] + after_big.rev
print("Yes")
print(ops.len)
for op in ops: print(op.span.lo + 1, op.span.hi)

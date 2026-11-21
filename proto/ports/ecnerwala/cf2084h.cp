// Codeforces 2084 H: Turtle and Nediam 2.
// Collapse s into maximal runs `blocks`. Any reachable string ends in a
// subsequence of alternating runs whose lengths grow by 1 per opposite-parity
// round. For each block parity (Even/Odd), a monotone `Staircase` maintains
// the newest block of height >= h at every h >= 1 while all steps grow by 1
// per round (`lift`), storing tops minus `lift` so they are never rewritten.
// Adding `floor + lift - 1` counts prefixes starting inside block 0 (for Even
// parity, `blocks.first!`) or block 1 (for Odd parity, `1`) before any pushed
// step; multiplying the final `dp` by `blocks.last!` chooses how many symbols
// of the last run remain.
@!multitest
use std.seq.{runs}

type M = Mod<1_000_000_007>

struct Staircase:
    steps: List<(top: Int, v: M)>
    lift: Int
    hidden: M

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while let cur = steps.last, cur.top <= top:
            steps.pop()
            if let b = steps.last: hidden -= M(b.top - cur.top) * b.v
        if let b = steps.last: hidden += M(b.top - top) * b.v
        steps.push((top:, v:))

    fn visible(self) -> M:
        if let cur = steps.last: hidden + M(cur.top + lift) * cur.v else: hidden

let n: 3..=2_000_000 = read()
let s: Str = read()
let blocks = [r.len for r in s.runs()]

if blocks.len == 1:
    print(M(n - 1))
else:
    domain Parity = 0..2
    var stairs = [Parity] _ => Staircase([], 0, 0)
    var dp: M = 1
    for (k, len) in blocks[..blocks.len - 1].items:
        let p = Parity(k % 2)
        stairs[p].lift += 1
        if k >= 1: stairs[p].push(len, dp)
        let floor = if p == Parity.first: blocks.first! else: 1
        dp = stairs[p].visible() + M(floor + stairs[p].lift - 1)
    print(dp * M(blocks.last!))

// The DP runs over maximal runs of s. For each parity of block index, the
// transition sums, over every height h >= 1, the dp of the newest block at
// least h tall, while all blocks grow by 1 per round: a monotone "staircase".
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
        steps.push((top: top, v: v))

    fn visible(self) -> M:
        if let cur = steps.last: hidden + M(cur.top + lift) * cur.v else: hidden

let n: 3..=2_000_000 = read()
let s: Str = read()
let blocks = [r.len for r in s.runs()]

if blocks.len == 1:
    print(M(n - 1))
else:
    var stairs = [0..2] _ => Staircase([], 0, 0)
    var dp: M = 1
    for k in 0..blocks.len - 1:
        let p = k % 2
        stairs[p].lift += 1
        if k >= 1: stairs[p].push(blocks[k], dp)
        let floor = if p == 0: blocks.first! else: 1
        dp = stairs[p].visible() + M(floor + stairs[p].lift - 1)
    print(dp * M(blocks.last!))

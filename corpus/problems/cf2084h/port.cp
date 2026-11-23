// Codeforces 2084 H: Turtle and Multiplication (run-length staircase DP).
// The DP runs over maximal runs `blocks` of `s`. Each round alternates parity
// of block index: for each height h >= 1, the transition sums the `dp` of the
// newest earlier block of the same parity at least h tall, while every earlier
// block grows by 1 per round (`lift`): a monotone `Staircase`.
//   - Heights below the newest step contribute `floor + lift - 1` base ways
//     (the first block of parity Even has `blocks.first` choices; Odd starts at 1).
//   - At the end, multiplying by `blocks.last` chooses the surviving length of
//     the final run.
@!multitest
use std.seq.runs

type M = Mod<1_000_000_007>
domain Parity = 0..2

// At height h the newest step at least h tall is visible. Every step grows by 1
// each round (`lift`), so tops are stored minus the lift and never rewritten.
struct Staircase:
    steps: List<(top: Int, v: M)>
    var lift: Int
    hidden: M

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while let cur = steps.last and cur.top <= top:
            steps.pop()
            if let b = steps.last: hidden -= M(b.top - cur.top) * b.v
        if let b = steps.last: hidden += M(b.top - top) * b.v
        steps.push((top, v))

    fn visible(self, floor: Int) -> M =
        hidden + M(floor + lift - 1) + (steps.last.map(s => M(s.top + lift) * s.v) ?? 0)

let n: 3..=2_000_000 = read()
let blocks = read<Str>().runs().map(r => r.len)
if blocks.len == 1:
    print(M(n - 1))
    return

var stairs = [Parity] _ => Staircase([], 0, 0)
var dp: M = 1
for k in 0..blocks.len - 1:
    let p = Parity(k % 2)
    stairs[p].lift += 1
    if k >= 1: stairs[p].push(blocks[k], dp)
    dp = stairs[p].visible(if p == Parity.first: blocks.first! else: 1)
print(dp * M(blocks.last!))

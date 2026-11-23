// Tuned: run lengths in one scan, the staircase as two flat lists, and the
// top-of-stack read by index instead of an optional copy of the last step.
@!multitest
type M = Mod<1_000_000_007>
domain Parity = 0..2

struct Staircase:
    tops: List<Int>
    vs: List<M>
    var lift: Int
    hidden: M

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while tops.len > 0 and tops[tops.len - 1] <= top:
            let cur = tops[tops.len - 1]
            tops.pop()
            vs.pop()
            if tops.len > 0: hidden -= M(tops[tops.len - 1] - cur) * vs[vs.len - 1]
        if tops.len > 0: hidden += M(tops[tops.len - 1] - top) * vs[vs.len - 1]
        tops.push(top)
        vs.push(v)

    fn visible(self, floor: Int) -> M:
        var r = hidden + M(floor + lift - 1)
        if tops.len > 0: r += M(tops[tops.len - 1] + lift) * vs[vs.len - 1]
        r

let n: 3..=2_000_000 = read()
let s: Str = read()
var blocks: List<Int> = []
var run = 1
for i in 1..s.len:
    if s[i] == s[i - 1]: run += 1
    else:
        blocks.push(run)
        run = 1
blocks.push(run)
if blocks.len == 1:
    print(M(n - 1))
    return

var stairs = [Parity] _ => Staircase([], [], 0, 0)
var dp: M = 1
let first = blocks[0]
for k in 0..blocks.len - 1:
    let p = Parity(k % 2)
    stairs[p].lift += 1
    if k >= 1: stairs[p].push(blocks[k], dp)
    dp = stairs[p].visible(if k % 2 == 0: first else: 1)
print(dp * M(blocks[blocks.len - 1]))

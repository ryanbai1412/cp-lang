@!multitest
type M = Mod<1_000_000_007>

struct Staircase:
    steps: List<(top: Int, v: M)>
    lift: Int
    hidden: M

    fn new() -> Staircase = Staircase([], 0, 0)

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

var blocks: List<Int> = []
var run = 1
for i in 1..n:
    if s[i] == s[i - 1]:
        run += 1
    else:
        blocks.push(run)
        run = 1
blocks.push(run)

if blocks.len == 1:
    print(M(n - 1))
else:
    var stairs: [0..2]Staircase = [0..2] _ => Staircase.new()
    var dp: M = 1
    for k in 0..blocks.len - 1:
        let z = k % 2
        stairs[z].lift += 1
        if k >= 1: stairs[z].push(blocks[k], dp)
        let floor = if z == 0: blocks[0] else: 1
        dp = stairs[z].visible() + M(floor + stairs[z].lift - 1)
    print(dp * M(blocks.last!))

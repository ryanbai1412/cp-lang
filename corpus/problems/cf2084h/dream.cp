// The DP runs over maximal blocks of s. For each parity of block index, the
// transition sums, over every height h ≥ 1, the dp of the newest block at least h
// tall, while all blocks grow by one per round: a monotone "staircase".
@!multitest
type M = Mod<1e9 + 7>

let n: 3..=2e6 = read()
let blocks = read<Str>().runs().map(r => r.len)
if blocks.len == 1: return print(M(n - 1))

// At height h the newest step at least h tall is visible. Every step grows by one
// each round (`lift`), so tops are stored minus the lift and never rewritten.
struct Staircase:
    var steps: List<(top: Int, v: M)> = []   // tops strictly decreasing
    var lift = 0
    var hidden: M = 0                        // Σ v·(top − next top), all but the newest step

    fn push(mut self, height: Int, v: M):
        let top = height - lift
        while let s = steps.pop_if(s => s.top <= top):
            if let b = steps.last: hidden -= M(b.top - s.top) * b.v
        if let b = steps.last: hidden += M(b.top - top) * b.v
        steps.push((top, v))

    fn visible(self) -> M =                  // Σ over heights h ≥ 1 of the visible v
        hidden + (steps.last.map(s => M(s.top + lift) * s.v) ?? 0)

var stairs: [0..2]Staircase = Staircase()
var dp: M = 1
for (k, len) in blocks.drop_last(1).items:
    let st = mut stairs[k % 2]
    st.lift += 1
    if k >= 1: st.push(len, dp)
    let floor = if k % 2 == 0: blocks.first! else: 1
    dp = st.visible() + M(floor + st.lift - 1)
print(dp * M(blocks.last!))

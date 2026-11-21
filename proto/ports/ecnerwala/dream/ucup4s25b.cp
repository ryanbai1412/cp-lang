// UCup 4 Stage 25 (Zagreb) B: balls roll left or right and merge on contact;
// a merged block moves the way the heavier side (ties: the left side) moved.
// Simulate with a stack of right-moving blocks: a left-moving block eats the
// stack top until it turns right or falls into the left wall. Every boundary
// b (between balls b - 1 and b) disappears exactly once: when the block
// starting at b merges leftward, or in the final flush, where everything
// still on the stack merges from the right. A query [l, r] ends as two
// blocks split at the boundary inside it that merged last.
enum Dir: Left, Right

let n: Int = read()
domain Ball = 0..n
type Cut = Ball.bounds
let balls: [Ball](w: 1..=1e9, side: Char) = read()
let dir = [Ball] i => if balls[i].side == 'R': Right else: Left
let mass = Prefix([Ball] i => Int(balls[i].w))

var merges: List<Cut> = []
var stack: List<(start: Cut, w: Int)> = []
for i in Ball:
    var (start, w, d) = (Ball.bounds.nth(i.ord), Int(balls[i].w), dir[i])
    while d == Left:
        merges.push(start)
        let top = stack.pop() ?? break
        d = if top.w >= w: Right else: Left
        (start, w) = (top.start, w + top.w)
    if d == Right: stack.push((start, w))
merges.push(Ball.bounds.last)
for block in stack.rev: merges.push(block.start)

var merged_at: [Cut]Int = 0
for (t, b) in merges.items: merged_at[b] = t
let last_merge = SparseTable(max, [Cut] b => (merged_at[b], b))

let q: Int = read()
repeat q:
    let (l, r): (1..=n, 1..=n) = read()
    let (lo, hi) = (Ball.bounds.nth(l - 1), Ball.bounds.nth(r))
    let (_, split) = last_merge.fold(lo..=hi)
    print(mass.sum(lo..split), mass.sum(split..hi))

// Source: The 4th Universal Cup, Stage 25 (Grand Prix of Zagreb), problem B (Big Balls),
//   qoj.ac/contest/3773/problem/18402
// limits: 4000 ms, 1024 MB
// Problem: n <= 1e6 balls on a line, each with weight w <= 1e9 rolling left or right at equal
//   speed; colliding balls merge, the sum moving as the heavier one (ties: right). For each of
//   q <= 1e6 subarrays [l, r] alone, print the total weight finally moving left and right.
// Solution: simulate the whole array with a stack of right-moving blocks: a left-moving block
//   eats the stack top until it turns right or reaches the left wall, and the final right
//   flush merges what is left. Each boundary between balls vanishes once; record when. A query
//   ends as two blocks split at its boundary that vanished last: a sparse table of
//   (time, boundary) and prefix sums give it, O((n + q) log n) with O(1) queries.
// Bounds: balls.w: 1..=1e9
enum Dir: Left | Right

let n: Int = read()
domain Ball = 0..n
type Cut = Ball.bounds
let balls: [Ball](w: 1..=1e9, side: Char) = read()
let dir = [Ball] i => if balls[i].side == 'R': Right else: Left
let mass = Prefix([Ball] i => Int(balls[i].w))

var merges: List<Cut> = []
var stack: List<(start: Cut, w: Int)> = []
for i in Ball:
    var (start, w, d) = (Ball.cuts.nth(i.ord), Int(balls[i].w), dir[i])
    while d == Left:
        merges.push(start)
        let top = stack.pop() ?? break
        d = if top.w >= w: Right else: Left
        (start, w) = (top.start, w + top.w)
    if d == Right: stack.push((start, w))
merges.push(Ball.cuts.last!)
for block in stack.rev: merges.push(block.start)

var merged_at: [Cut]Int = 0
for (t, b) in merges.items: merged_at[b] = t
let last_merge = SparseTable(max, [Cut] b => (merged_at[b], b))

let q: Int = read()
repeat q:
    let (l, r): (1..=n, 1..=n) = read()
    let (lo, hi) = (Ball.cuts.nth(l - 1), Ball.cuts.nth(r))
    let (_, split) = last_merge.fold(lo..=hi)
    print(mass.fold(lo..split), mass.fold(split..hi))

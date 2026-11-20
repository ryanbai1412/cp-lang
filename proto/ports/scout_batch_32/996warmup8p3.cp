// 996 Warmup 8 P3 (DMOJ: 996warmup8p3) - balls bouncing between two walls
// Ideal v2 port. original: corpus/submissions/5324670.cpp
//
// n balls move at unit speed on a segment of length L, bouncing off both walls
// (collisions between balls are pass-throughs). Print the time at which the
// left wall has taken kl hits and the right wall kr hits, measured half a
// period after the later wall is done.
//
// Hits by time t are monotone in t, so `bsearch_first` finds the moment one
// wall finishes; the other wall's remaining hits come from sorting the balls
// still heading for it.
const L = 1_000_000_001
enum Wall = Left | Right
let (n, kl, kr): (1..=1e5, 1..=1e9, 1..=1e9) = read()
let pos: [0..n](0..=L) = read()
let dir: [0..n]Bool = read()            // true = moving right

type Ball = (at: Int, right: Bool)
type State = (hits: [Wall]Int, balls: List<Ball>)
fn mirror(b: Ball) -> Ball = (L - b.at, not b.right)

// Hits on each wall and every ball's state after t seconds.
fn after(t: Int) -> State:
    let (laps, m) = (t / (2 * L), t % (2 * L))
    var hits: [Wall]Int = n * laps
    var balls: List<Ball> = []
    // A left-mover at p: reaches the left wall at p, the right wall at p + L.
    fn left_mover(p: Int, own: Wall) -> Ball:
        if m < p: return (p - m, false)
        if m < p + L:
            hits[own] += 1
            return (m - p, true)
        hits[Left] += 1
        hits[Right] += 1
        (2 * L - m + p, false)
    for (p, right) in zip(pos, dir):
        balls.push(if right: mirror(left_mover(L - p, Right)) else: left_mover(p, Left))
    (hits, balls)

fn done(h: [Wall]Int) -> [Wall]Bool = [Wall] w => h[w] >= (if w == Left: kl else: kr)

// WISH X7: the search needs the simulated state at the answer, so the
// predicate becomes an optional-returning helper for `bsearch_first_some`;
// with plain `bsearch_first` the winning `after(t)` is simulated twice.
fn first_done(t: Int) -> ?State:
    let s = after(t)
    if any(done(s.hits)): s else: none

let (t1, (hits, balls)) = bsearch_first_some(0..=4e18, first_done)!
let fin = done(hits)
if all(fin):
    print(t1 + L)
else:
    // Count what the unfinished wall still needs among balls heading for it.
    let (wall, need) = if fin[Left]: (Right, kr - hits[Right]) else: (Left, kl - hits[Left])
    let toward = [if wall == Right: b else: mirror(b) for b in balls]
    let movers = sorted([b.at for b in toward if b.right], by: p => desc(p))
    print(t1 + 2 * L - (if need < movers.len: movers[need - 1] else: 0))

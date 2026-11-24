// Source: 996 Warmup 8 P3, https://dmoj.ca/problem/996warmup8p3
// Problem: n <= 1e5 balls move at unit speed on a segment of length L = 1e9 + 1, ball i from
//   position p_i in direction d_i (1 = right), bouncing off both walls and passing through each
//   other. Print the time at which the left wall has taken kl <= 1e9 hits and the right wall
//   kr <= 1e9, measured half a period after the later wall is done.
// Solution: every 2L seconds each ball hits each wall once, and within a period ball i first
//   reaches each wall at a fixed offset, so the hits by time t are a count of offsets <= t mod
//   2L. The hits only grow, so `first_true` finds the moment t1 one wall is done; if the other
//   is not, its remaining hits come from the balls heading for it at t1, nearest first.
//   O(n log(kl / n)).
// original: corpus/submissions/5324670.cpp
// Bounds: n: 1..=1e5; kl, kr: 1..=1e9; pos: 0..=L; dir: 0..=1
const L = 1_000_000_001
let (n, kl, kr): (1..=1e5, 1..=1e9, 1..=1e9) = read()
domain Ball = 0..n
let pos: [Ball]0..=L = read()
let dir: [Ball]0..=1 = read()

// when, within a period, each ball first reaches each wall
let reach: [Ball](left: Int, right: Int) = [Ball] b =>
    if dir[b] == 1: (left: 2 * L - pos[b], right: L - pos[b]) else: (left: pos[b], right: pos[b] + L)

fn hits(t: Int) -> (left: Int, right: Int):
    let (laps, m) = (t / (2 * L), t % (2 * L))
    (left: n * laps + count(reach[b].left <= m for b in Ball),
     right: n * laps + count(reach[b].right <= m for b in Ball))

// where ball b is, and whether it heads right, at m seconds into a period
fn place(b: Ball, m: Int) -> (at: Int, right: Bool):
    let flip = dir[b] == 1
    let p = if flip: L - pos[b] else: pos[b]
    let s = if m < p: (at: p - m, right: false) elif m < p + L: (at: m - p, right: true)
        else: (at: 2 * L - m + p, right: false)
    if flip: (at: L - s.at, right: not s.right) else: s

let t1 = (0..=2e9 * L / n + 2).first_true(t => hits(t).left >= kl or hits(t).right >= kr)!
let h = hits(t1)
if h.left >= kl and h.right >= kr:
    print(t1 + L)
else:
    // seen from the unfinished wall, the balls heading for it, nearest first
    let to_right = h.right < kr
    let need = if to_right: kr - h.right else: kl - h.left
    let movers = [if to_right: s.at else: L - s.at
                  for b in Ball for s in [place(b, t1 % (2 * L))] if s.right == to_right]
        .sorted(by: p => desc(p))
    print(t1 + 2 * L - (if need < movers.len: movers[need - 1] else: 0))

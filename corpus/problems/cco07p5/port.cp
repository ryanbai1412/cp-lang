// Source: CCO '07 P5 (Particle Catcher), https://dmoj.ca/problem/cco07p5
// Problem: interactive. A particle moves v steps per query round a ring of N positions
//   (2 <= N <= 1e5). `GetSize` gives N; `Look a b` says whether it is in a..b now, then it
//   moves; finish with `Guess v p`, its velocity and position. At most 1e5 queries.
// Solution: cut the ring into blocks of b = clamp(2e6 / N, 2, N) and `Look` at a random
//   block until one hits (each position is seen with chance b / N, so about N / b <= 5000
//   looks); then the next position and the velocity are among bN <= 2e6 candidates
//   (x + v, v). Halve the candidates with `Look 0 mid` at their median
//   position, keep the consistent ones and advance each by its own v, until one remains:
//   O(log bN) more looks, O(bN log bN) work in all as the candidates halve.
// original: corpus/submissions/5120565.cpp
use std.rand.{Rng}

let n: 2..=1e5 = ask("GetSize")
let block = min(n, max(2, 2_000_000 / n))

fn look(l: Int, r: Int) -> Bool:
    let answer: Str = ask("Look", l, r)
    answer == "True"

var rng = Rng.new(1412)
fn random_block() -> Int = rng.int(0, (n - 1) / block) * block

var lo = random_block()
while not look(lo, min(n, lo + block) - 1): lo = random_block()

type Guess = (p: Int, v: Int)  // where the particle is now, and its velocity
var cands: List<Guess> = [(p: (x + v) % n, v:) for v in 0..n for x in lo..min(n, lo + block)]
while cands.len > 1:
    cands.sort()
    let mid = cands[(cands.len - 1) / 2].p
    let hit = look(0, mid)
    cands = [(p: (c.p + c.v) % n, v: c.v) for c in cands if (c.p <= mid) == hit]

let last = cands[0]
print("Guess", last.v, last.p)

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
let n: 2..=1e5 = ask("GetSize")
let bsize = min(n, max(2, 2_000_000 / n))

fn look(l: Int, r: Int) -> Bool:
    let tok: Str = ask("Look", l, r)
    tok == "True"

var rng_state = 1412
fn next_rand(bound: Int) -> Int:
    rng_state = (rng_state * 1_103_515_245 + 12_345) % 2_147_483_648
    rng_state % bound

struct State:
    p: Int
    v: Int

let blocks = (n + bsize - 1) / bsize
var (lo, hi) = (0, 0)
while true:
    lo = next_rand(blocks) * bsize
    hi = min(n, lo + bsize)
    if look(lo, hi - 1): break

var cands: List<State> = []
for v in 0..n:
    for x in lo..hi:
        cands.push(State((x + v) % n, v))

while cands.len > 1:
    cands.sort()
    let mid = cands[(cands.len - 1) / 2].p
    let hit = look(0, mid)
    var nxt: List<State> = []
    for idx in 0..cands.len:
        let c = cands[idx]
        if (hit and c.p <= mid) or (not hit and c.p > mid):
            nxt.push(State((c.p + c.v) % n, c.v))
    cands = nxt

let final_st = cands[0]
print("Guess", final_st.v, final_st.p)

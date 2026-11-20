// CCO '07 P5 — UFO / Moving Target on a Ring (Interactive) (today form for proto/cpc)
// original: corpus/submissions/5120565.cpp
let n: 2..=1000 = ask("GetSize")
let BSIZE = 2

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

var x = 0
while true:
    x = next_rand(n - BSIZE + 1)
    if look(x, x + BSIZE - 1): break

var cands: List<State> = []
for v in 0..n:
    for j in 0..BSIZE:
        cands.push(State((x + v + j) % n, v))

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

// Source: Codeforces 2135 D1 (From the Unknown, easy version),
//   https://codeforces.com/contest/2135/problem/D1
// limits: 3000 ms, 500 MB
// Problem: interactive, t <= 10. An editor of hidden width W <= 1e5 puts words greedily on
//   lines of total length <= W. Query "? n a_1 .. a_n" (n, a_i <= 1e5) returns the number of
//   lines, or 0 if some a_i > W. Find W with at most 2 queries and print "! W".
// Solution: first send 1e4 words of length 125. Answer 0 means W < 125: send 15000 ones,
//   ceil(15000 / W) is distinct for each W < 125. Otherwise floor(W / 125) words fit a line,
//   which pins W to [lo, hi] with hi - lo < 25000; send pairs (lo, i - lo) for i in lo+1..hi,
//   each on one line iff i <= W, so the line count gives W directly.

@!interactive

const WMAX = 100_000
const G = 125
const Q1 = 10_000
const Q02 = 15_000

fn ask(vals: List<Int>) -> Int:
    print("?", vals.len, vals)
    read()

@!multitest
let resp1 = ask(List.filled(Q1, G))
if resp1 == 0:
    let resp2 = ask(List.filled(Q02, 1))
    print("!", (1..=Q02).find(ans => resp2 == ceil_div(Q02, ans))!)
    return

let w_min = ceil_div(Q1, resp1) * G
let w_max = min(WMAX, (Q1 - 1) / (resp1 - 1) * G + G - 1)
let q = [x for i in w_min + 1..=w_max for x in [w_min, i - w_min]]
let resp2 = ask(q)
print("!", (w_max - w_min) + w_max - resp2)

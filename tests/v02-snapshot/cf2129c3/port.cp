// Source: Codeforces 2129C3 (Interactive RBS (Hard Version)),
//   https://codeforces.com/contest/2129/problem/C3
// Problem: interactive, multitest. A hidden bracket string s of length n <= 1000 with at least
//   one '(' and one ')'. A query lists up to 1000 indices (repeats allowed) and returns the
//   number of balanced substrings of that sequence; recover s in at most 100 queries.
// Solution: "some prefix has a balanced substring" is monotone, so a binary search finds an
//   index r with s[r-1] = '(' (or, if none, s ends with '('): a known open bracket o. Then each
//   query tests 13 positions at once: k copies of "o p" then one o add k(k+1)/2 balanced
//   substrings iff s[p] = ')'. The k are picked so each weight beats the sum of the smaller
//   ones within 1000 indices, so the count decodes greedily, largest first. About 77 + 10 queries.

@!interactive

const MAX_QUERY_LEN = 1000
fn tri(k: Int) -> Int = k * (k + 1) / 2

fn build_weights(budget: Int) -> List<Int>:
    var chosen: List<Int> = []
    var (rem_len, sum_w) = (budget, 0)
    for k in 1..budget:
        let cost = 2 * k + 1
        if cost > rem_len: break
        if tri(k) > sum_w:
            chosen.push(k)
            rem_len -= cost
            sum_w += tri(k)
    chosen

let weights = build_weights(MAX_QUERY_LEN)
let blk = weights.len

@!multitest
let n: 2..=1000 = read()
domain Pos = 1..=n

fn ask(inds: List<Pos>) -> Int:
    print("?", inds.len, inds)
    read()

let open_i = (Pos(2)..=Pos.last!).first_true(
    p => ask([x for x in Pos if x <= p]) > 0,
).map(r => r.prev!) ?? Pos.last

var ans: [Pos]Char = '?'
for start in 0..n by blk:
    let inds = [Pos(min(start + z + 1, n)) for z in 0..blk]
    var query: List<Pos> = []
    for (pos, k) in zip(inds, weights):
        repeat k:
            query.push(open_i!)
            query.push(pos)
        query.push(open_i!)

    var rem = ask(query)
    for (pos, k) in zip(inds, weights).rev:
        let w = tri(k)
        if rem >= w:
            ans[pos] = ')'
            rem -= w
        else:
            ans[pos] = '('

print("!", ans)

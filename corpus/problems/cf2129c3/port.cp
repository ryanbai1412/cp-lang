// Codeforces 2129 C3: Interactive RBS (Hard Version).
// Recover a bracket string `s` of length `n` (`n <= 1000`, containing at least
// one `(` and one `)`) using queries of length at most `MAX_QUERY_LEN = 1000`
// that return the number of balanced substrings of the queried subsequence.
//
// Phase 1: `ask(1..=p) > 0` is monotone in `p`. Its first true position `r`
// must have `s[r - 1] = '('` and `s[r] = ')'`; if no prefix has a balanced
// substring, `s` is `)))...(((` so `Pos.last` is `'('`.
// Phase 2: build superincreasing triangular weights `tri(k) = k(k + 1) / 2`:
// repeating `(open_i, pos)` `k` times followed by a separator `open_i` adds
// `tri(k)` balanced substrings iff `s[pos] == ')'` (and `0` if `'('`), using
// `2k + 1` query indices. Requiring `tri(k) > sum(prev)` makes the subset sum
// uniquely decodable greedily from largest `k` down, fitting `blk = 13`
// brackets per query under `MAX_QUERY_LEN = 1000`. The last partial block pads
// remaining slots with `Pos.last`, harmlessly re-decoding `ans[Pos.last]`.

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

let open_i = (Pos(2)..=Pos.last!).first_true(p => ask([x for x in Pos if x <= p]) > 0,
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

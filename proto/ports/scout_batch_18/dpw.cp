// AtCoder DP W - Intervals (DMOJ: dpw, 20pt, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Choose a binary string of length n. Each rule (l, r, a) adds a to the score
// if the string has at least one '1' in [l, r]. Maximize total score.
//
// Sweep left-to-right over positions i = 1..=n. Maintain dp[j] for j in 0..=n:
// the best score for prefix 1..i whose last '1' is at j (j = 0 means all '0's).
// At step i:
//   1) Placing '1' at i sets dp[i] = max_{0 <= j < i} dp[j].
//   2) Every rule (l, i, a) ending at i adds a to dp[l..=i].
// Answer is max_{0 <= j <= n} dp[j] (dp[0] stays 0, so answer >= 0).
use std.lazy.{LazySegTree, MaxOpt, AddMax}

let (n, m): (1..=2e5, 1..=2e5) = read()
domain Pos = 0..=n
domain Rule = 0..m
// WISH W1: cross-field ordering `where l <= r` inside a record read array
let rules: [Rule](l: 1..=n, r: 1..=n, a: -1e9..=1e9) = read() where all(x.l <= x.r for x in rules)

var by_r: [Pos]List<(l: Pos, a: Int)> = []
for k in Rule:
    by_r[Pos(rules[k].r)].push((l: Pos(rules[k].l), a: rules[k].a))

fn solve() -> Int:
    var st = LazySegTree<Pos, ?Int, Int>.new(MaxOpt, AddMax)
    st.set(Pos(0), 0)
    for i in Pos(1)..=Pos(n):
        let best_prev = st.fold(Pos(0)..i)!
        st.set(i, best_prev)
        for seg in by_r[i]:
            st.apply(seg.l..=i, seg.a)
    st.fold(Pos(0)..=Pos(n))!

@brute(solve)
fn solve_brute() -> Int
    where n <= 16:
    var best = 0
    for mask in 0..(1 << n):
        var score = 0
        for k in Rule:
            let (l, r, a) = (rules[k].l, rules[k].r, rules[k].a)
            var has_one = false
            for p in l..=r:
                if ((mask >> (p - 1)) & 1) == 1:
                    has_one = true
                    break
            if has_one: score += a
        best max= score
    best

print(solve())

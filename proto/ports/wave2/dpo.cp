// AtCoder DP O Matching: n men and n women; ok[m, w] says whether man m and
// woman w can pair. Count the perfect matchings.
type Z = Mod<1_000_000_007>
let n: 1..=21 = read()
domain Man = 0..n
domain Woman = 0..n
type Taken = Subset<Woman>
let ok: [Man, Woman]0..=1 = read()

// ways[s]: matchings of the first |s| men with exactly the women in s; the
// last of those men takes one of them.
var ways: [Taken]Z = 0
ways[Taken.empty] = 1
for s in Taken if not s.is_empty:
    let man = Man.from_idx(s.len - 1)!
    for w in s if ok[man, w] == 1: ways[s] += ways[s.without(w)]
print(ways[Taken.full])

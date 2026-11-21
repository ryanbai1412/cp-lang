// cp: 0.1 0f8d77b
// AtCoder DP O, Matching: n men and n women, compatible[i, j] says whether
// man i and woman j can pair. Count the perfect matchings.
const P = 1000000007
type Z = Mod<P>
let n: 1..=21 = read()
domain Man = 0..n
domain Woman = 0..n
type Taken = Subset<Woman>
let compatible: [Man, Woman]Bool = read()

// ways[s]: matchings of the first |s| men with exactly the women in s.
// The last of those men takes one of the women in s.
var ways: [Taken]Z = Z.zero
ways[Taken.empty] = Z.one
for s in Taken:
    if not s.is_empty:
        let man = Man.nth(s.len - 1)
        ways[s] = sum(ways[s.without(w)] for w in s if compatible[man, w])
print(ways[Taken.full])

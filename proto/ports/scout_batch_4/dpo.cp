// Educational DP O, Matching: count the ways to pair n men with n women so
// that every pair is compatible.
// original: corpus/submissions/4131780.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=21 = read()
domain Man = 0..n
domain Woman = 0..n
type Taken = Subset<Woman>
let likes: [Man, Woman]Bool = read()

// ways[t]: pairings of the first t.len men with exactly the women in t.
// The last of those men takes one of them.
var ways: [Taken]Z = 0
ways[Taken.empty] = 1
for t in Taken:
    if not t.is_empty:
        let m = Man(t.len - 1)
        ways[t] = sum(ways[t.without(w)] for w in t if likes[m, w])
print(ways[Taken.full])

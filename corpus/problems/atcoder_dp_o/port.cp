// Source: AtCoder Educational DP Contest O (Matching), https://atcoder.jp/contests/dp/tasks/dp_o
// limits: 2000 ms, 1024 MB
// Problem: n <= 21 men and n women with a 0/1 compatibility matrix. Count the ways to split
//   everyone into n compatible man-woman pairs, mod 1e9+7.
// Solution: hand out women to the men in order: ways[t], over subsets t of women, counts the
//   pairings of the first |t| men with exactly the women in t. The last of those men takes
//   some woman w in t he likes, so ways[t] = sum of ways[t without w]. O(2^n n).
// original: corpus/submissions/4131780.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=21 = read()
domain Man = 0..n
domain Woman = 0..n
type Taken = Subset<Woman>
let likes: [Man, Woman]0..=1 = read()

// ways[t]: pairings of the first t.len men with exactly the women in t.
// The last of those men takes one of them.
var ways: [Taken]Z = 0
ways[Taken.empty] = 1
for t in Taken:
    if t != Taken.empty:
        let m = Man(t.len - 1)
        ways[t] = sum(ways[t.without(w)] for w in t if likes[m, w] == 1)
print(ways[Taken.full])

// Source: Mock CCC '22 2 S2 (Kaguya Wants to Grow the Student Council),
//   https://dmoj.ca/problem/nccc10s2
// limits: 250 ms, 1024 MB
// Problem: N <= 1e4 ballots, each a permutation of the first K <= 26 uppercase letters. Pick
//   the largest ranked council such that for any two members every ballot ranks them in the
//   council's order; print its size.
// Solution: ahead[x, y] holds when x precedes y on every ballot (cleared by each ballot's
//   pairs). That relation is a DAG and a council is a chain in it, so the answer is the
//   longest path, memoised per letter: O(N * K^2) to build, O(K^2) to search.
// Bounds: n: 1..=1e4; k: 1..=26
// original: corpus/submissions/4214203.cpp
let (n, k): (1..=1e4, 1..=26) = read()
domain L = 'A'..'A' + k
let ballots: [n]Str = read()

var ahead: [L, L]Bool = true  // ahead[x, y]: x before y on every ballot
for b in ballots:
    for (x, y) in b.pairs():
        ahead[L(y), L(x)] = false

@memo
fn chain(u: L) -> Int = 1 + (max(chain(v) for v in L if v != u and ahead[u, v]) ?? 0)

print(max(chain(u) for u in L)!)

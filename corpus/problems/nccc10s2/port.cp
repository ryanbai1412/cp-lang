// Source: Mock CCC '22 2 S2 (Kaguya Wants to Grow the Student Council),
//   https://dmoj.ca/problem/nccc10s2
// limits: 250 ms, 1024 MB
// Problem: N <= 1e4 ballots, each a permutation of the first K <= 26 uppercase letters. Pick
//   the largest ranked council such that for any two members every ballot ranks them in the
//   council's order; print its size.
// Solution: ahead[x, y] holds when x precedes y on every ballot (cleared by each ballot's
//   pairs). That relation is a DAG and a council is a chain in it, so the answer is the
//   longest path, memoised per letter: O(N * K^2) to build, O(K^2) to search.
// cp: 0.1 8ef64b9+dirty
let n: 1..=1e4 = read()
let k: 1..=26 = read()
domain L = 'A'..'A' + k
let orders: [n]Str = read()  // WISH: read each word as a `Perm<L>` (filed with stdlib)

var ahead: [L, L]Bool = true  // ahead[x, y]: x before y in every order
for o in orders:
    for (x, y) in o.pairs():
        ahead[L(y), L(x)] = false

@memo
fn chain(u: L) -> Int:
    ([chain(v) for v in L if v != u and ahead[u, v]].max() ?? 0) + 1

print([chain(u) for u in L].max()!)

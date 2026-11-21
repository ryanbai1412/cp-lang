// Codeforces 2245 D1: Construct an Array (Easy Version).
// The pair (v, v) fixes each sign. Two non-negatives always sum to >= 0 and two
// negatives to < 0, while a non-negative u and a negative v sum to >= 0 iff
// a_u >= |a_v|, so every u beats the negatives of smallest magnitude. Rank each
// negative by how many u beat it (magnitude 1 + the number beaten more often), give
// each u the number of negatives it beats, and check all restrictions against this
// one candidate. O(n^2).
@!multitest
domain V = read(1..=1000)
let m: Int = read()                                    // n(n + 1) / 2: each pair once
var fine: [Sym<V>]Bool = false                         // fine[u, v] is fine[v, u]
for (o, i, j) in read<[m](1..=2, V, V)>(): fine[i, j] = o == 1

let neg = V.filter(v => not fine[v, v])
let beaten = [V] v => V.count(u => fine[u, u] and fine[u, v])
let a = [V] v =>
    if fine[v, v]: neg.count(w => fine[v, w])
    else: -(1 + neg.count(w => beaten[w] > beaten[v]))

if all(fine[u, v] == (a[u] + a[v] >= 0) for (u, v) in Sym<V>):
    print("YES")
    print(a)
else:
    print("NO")

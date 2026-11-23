// Source: Codeforces 2222E (Seek the Truth), https://codeforces.com/contest/2222/problem/E
// Problem: interactive, multitest (t <= 1e4, 2 <= n <= 60). Hidden k in {AND, OR, XOR} and a
//   mask 1 <= c < 2^n; f(x) = x k c. Pick a start a for a set S = {a}; "I x" inserts f(x) and
//   returns |S|, "Q y" counts z in S with z >= y. Within n + 3 interactions print "A k c".
// Solution: start at 0; "I 0" gives |S| = 1 only for AND, which "I 2^n-1" then turns to
//   S = {0, c}. With S = {0, c}, n Q queries read c bit by bit from the top. One last insert
//   (u = c & (c - 1)) or query (a 2-bit supermask of c, for powers of two) tells OR from XOR.
// 1. Identify `k == 1` vs `{2, 3}`: `ask('I', 0)` inserts `0 & v == 0`
//    (leaving `|S| == 1`) when `k == 1`, or `0 | v == 0 ^ v == v > 0`
//    (giving `|S| == 2`) when `k in {2, 3}`. If `k == 1`, inserting
//    `(1 << n) - 1` adds `((1 << n) - 1) & v == v`, so `S == {0, v}` in all
//    three cases.
// 2. Recover `v`: since `S == {0, v}`, `ask('Q', cand) > 0` holds iff
//    `v >= cand`. Testing `cand = v + (1 << b)` from `b = n - 1` down to `0`
//    recovers every bit of `v` by binary search.
// 3. Distinguish `k == 2` (OR) from `k == 3` (XOR) when `S == {0, v}`:
//    - If `v` is not a power of 2, `u = v & (v - 1)` is a non-zero strict
//      submask of `v`: `u | v == v` keeps `|S| == 2`, whereas `u ^ v` is the
//      lowest set bit of `v` (not in `{0, v}`), raising `|S|` to `3`.
//    - If `v` is a power of 2, `nv = if v == 1: 3 else: v + 1` is a 2-bit
//      strict supermask of `v`: `nv | v == nv` gives `ask('Q', nv) == 1`,
//      whereas `nv ^ v == nv - v < nv` leaves no element `>= nv` (`0 != 1`).

@!interactive

fn ask(kind: Char, x: Int) -> Int:
    print(kind, x)
    read()

@!multitest
let n: 2..=60 = read()
print(0)

let n_sz = ask('I', 0)
if n_sz == 1: ask('I', (1 << n) - 1)

var v = 0
for b in (0..n).rev:
    if ask('Q', v + (1 << b)) > 0: v += 1 << b

let k = if n_sz == 1:
    1
elif not v.is_pow2:
    if ask('I', v & (v - 1)) == 3: 3 else: 2
else:
    let nv = if v == 1: 3 else: v + 1
    ask('I', nv)
    if ask('Q', nv) != 1: 3 else: 2

print("A", k, v)

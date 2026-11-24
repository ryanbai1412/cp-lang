// Source: Codeforces 2201A1 (Lost Civilization, Easy Version),
//   https://codeforces.com/contest/2201/problem/A1
// limits: 2000 ms, 256 MB
// Problem: a sequence grows by inserting x + 1 right after some element x. Given the result a
//   (n <= 3e5, a_i <= 1e9), print the length of the shortest sequence that can produce it.
//   Multitest, sum of n <= 3e5.
// Solution: undo insertions from the largest value down, ties left to right: an element goes
//   when the live element just before it is one smaller (everything larger between them has
//   already gone). Keep the live elements in a doubly linked list; the answer is n minus the
//   removals. O(n log n) for the sort.
// Bounds: n: 1..=300_000; a: 1..=10 ** 9

@!multitest
let n: 1..=300_000 = read()
let a: [0..n]1..=10 ** 9 = read()

var prv: [0..n]?Int = [0..n] i => if i > 0: i - 1 else: None
var nxt: [0..n]?Int = [0..n] i => if i + 1 < n: i + 1 else: None
var ans = n
for i in (0..n).sorted(by: i => (-a[i], i)):
    if let p = prv[i] and a[p] == a[i] - 1:
        ans -= 1
        let nx = nxt[i]
        nxt[p] = nx
        if let q = nx: prv[q] = p
print(ans)

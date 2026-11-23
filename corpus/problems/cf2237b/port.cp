// Source: Codeforces 2237B (Annoying the Ghost), https://codeforces.com/contest/2237/problem/B
// Problem: piles a_1..a_n and a strictly increasing target b_1..b_n (values <= 1e9). Add any
//   non-negative amount to each pile, then swap adjacent piles; print the fewest swaps that make
//   the row exactly b, or -1. Multitest, sum of n <= 2000.
// Solution: a pile of size x can end at position j iff b_j >= x, i.e. at any j >= slot(x), the
//   first such j (none: -1). Fill positions left to right: bubble the nearest remaining pile with
//   slot <= i to i, paying its distance; this keeps the others in order, and an exchange argument
//   makes it optimal. O(n^2).

@!multitest
let n: 1..=5_000 = read()
let a_in: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
var slots: [0..n]?Int = [0..n] i => (0..n).first_true(j => b[j] >= a_in[i])

var ans = 0
for i in 0..n:
    let j = (i..n).find(j => if let s = slots[j]: s <= i else: false) else:
        print(-1)
        return
    // Hand shift instead of `slots.insert(i, slots.remove(j))` (dream:23):
    // `List.insert`/`List.remove` fall back to the slower interpreter path (see friction).
    let x = slots[j]
    for k in (i + 1..=j).rev: slots[k] = slots[k - 1]
    slots[i] = x
    ans += j - i
print(ans)

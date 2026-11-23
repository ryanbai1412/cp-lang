// Codeforces 2222 B — Parity-Split Greedy Removals
//
// Each odd `x in xs` (`cnt0 = count(x % 2 == 1)`) operates on the even-indexed
// group `a[0, 2, ...]`, and each even `x` (`m - cnt0`) operates on the
// odd-indexed group `a[1, 3, ...]`. Within each group, if `cnt > 0`:
// - The first operation must remove one element; dropping the largest element
//   (`s.pop()`, even if negative) minimizes the remaining sum.
// - Each of the remaining `cnt - 1` operations can optionally drop another
//   element whenever it is positive (`x > 0`), since dropping non-positive
//   elements would increase or leave unchanged the remaining sum.

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
let xs: [0..m]1..=10 ** 9 = read()

let cnt0 = xs.count(x => x % 2 == 1)
fn best_rem(vals: List<Int>, var cnt: Int) -> Int:
    var s = vals.sorted()
    if cnt > 0:
        cnt -= 1
        s.pop()
        while let x = s.last and cnt > 0 and x > 0:
            cnt -= 1
            s.pop()
    s.sum()

print(best_rem([a[i] for i in 0..n by 2], cnt0) + best_rem([a[i] for i in 1..n by 2], m - cnt0))

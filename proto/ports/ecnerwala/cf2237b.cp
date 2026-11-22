// Codeforces 2237 B — Minimum Adjacent Swaps to Dominate
//
// The target array `b` is sorted non-decreasingly, so each `x = a_in[i]` can
// legally occupy any position `>= slot(x) = bsearch_first(0..n, j => b[j] >= x)`
// (`none` if `x > b[n - 1]`).
//
// Filling positions `i = 0..n` left-to-right, any element whose slot is `<= i`
// is legal at `i`; bubbling the earliest such element `j >= i` to `i` uses
// `j - i` adjacent swaps and preserves the relative order of all remaining
// elements, which is optimal by an exchange argument (`O(n^2)` for `n <= 5000`).

@!multitest
let n: 1..=5_000 = read()
let a_in: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
var slots: List<?Int> = [bsearch_first(0..n, j => b[j] >= x) for x in a_in]

var ans = 0
for i in 0..n:
    let j = (i..n).first(j => if let s = slots[j]: s <= i else: false) else:
        print(-1)
        return
    slots.insert(i, slots.remove(j))
    ans += j - i
print(ans)

// EGOI '23 P6 (Candy): fewest adjacent swaps that bring the first m boxes to
// a total of at least k.
// original: corpus/submissions/5678056.cpp
(n, m, k): (1..=100, 1..=n, 1..=1e11) = read()
let a: [0..n]0..=1e9 = read()
let cap = n * n / 2 + 10

// most candy from boxes j.. placed into slots i..m with at most s swaps
@memo
fn best(i: 0..=m, j: 0..=n, s: 0..cap) -> Int:
    if i == m or j == n: return 0
    let skip = best(i, j + 1, s)
    if s < j - i: return skip
    return max(skip, a[j] + best(i + 1, j + 1, s - (j - i)))

if let s = bsearch_first(0..cap, s => best(0, 0, s) >= k): print(s)
else: print("NO")

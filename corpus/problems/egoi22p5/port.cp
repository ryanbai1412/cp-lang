// Source: EGOI 2022 P5 (Data Centers), https://dmoj.ca/problem/egoi22p5
// Problem: n <= 1e5 data centres with at most 1e9 free machines each, and s <= 5000 services
//   in order: service i sorts the centres by free machines, descending, and takes m_i machines
//   from each of the top c_i (always enough). Print the final counts in descending order.
// Solution: keep the counts sorted. After taking m from the top c, both the top c less m
//   and the rest are still sorted, so std `merge_sorted` on the window where they overlap
//   (two binary searches) restores the order instead of a sort. O(n log n + s n).
// Bounds: n: 1..=1e5; s: 1..=5000; start: 0..=1e9
// original: corpus/submissions/5203693.cpp

let n: 1..=1e5 = read()
let s: 1..=5000 = read()
let start: [n]0..=1e9 = read()

var free: List<Int> = start.sorted()  // ascending, so the top centres are the tail
for _ in 0..s:
    let take: 1..=1e9 = read()
    let top: 1..=n = read()
    let cut = n - top
    for i in cut..n: free[i] -= take
    // both runs are still sorted; only rest values above the smallest taken one and taken
    // values below the largest rest one can be out of place
    if cut > 0:
        let lo = (0..cut).first_true(i => free[i] > free[cut]) ?? cut
        let hi = (cut..n).first_true(i => free[i] >= free[cut - 1]) ?? n
        let merged = free[lo..cut].merge_sorted(free[cut..hi])
        for k in 0..merged.len: free[lo + k] = merged[k]
print(free.rev)

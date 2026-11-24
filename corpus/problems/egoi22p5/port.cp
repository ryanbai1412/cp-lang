// Source: EGOI 2022 P5 (Data Centers), https://dmoj.ca/problem/egoi22p5
// Problem: n <= 1e5 data centres with at most 1e9 free machines each, and s <= 5000 services
//   in order: service i sorts the centres by free machines, descending, and takes m_i machines
//   from each of the top c_i (always enough). Print the final counts in descending order.
// Solution: keep the counts sorted. After taking m from the top c, both the top c less m
//   and the rest are still sorted, so one two-pointer merge restores the order instead of
//   a sort (std `merge_sorted`). O(n log n + s n).
// Bounds: n: 1..=1e5; s: 1..=5000; start: 0..=1e9
// original: corpus/submissions/5203693.cpp

let n: 1..=1e5 = read()
let s: 1..=5000 = read()
let start: [n]0..=1e9 = read()

var free: List<Int> = start.sorted()  // ascending, so the top centres are the tail
for _ in 0..s:
    let take: 1..=1e9 = read()
    let top: 1..=n = read()
    // the rest and the top `top` less `take` are both still ascending
    let cut = n - top
    free = free[0..cut].merge_sorted([x - take for x in free[cut..n]])
print(free.rev)

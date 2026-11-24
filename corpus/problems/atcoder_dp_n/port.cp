// Source: AtCoder Educational DP Contest N (Slimes), https://atcoder.jp/contests/dp/tasks/dp_n
// limits: 2000 ms, 1024 MB
// Problem: n <= 400 slimes in a row with sizes a_i <= 1e9. Merging two adjacent slimes costs
//   the sum of their sizes and leaves one slime of that size. Print the least total cost to
//   merge everything into one slime.
// Solution: the last merge on a range r joins its two halves at some split, so
//   cost[r] = min over splits (x, y) of cost[x] + cost[y], plus the total size of r.
//   `I.ranges` goes by length, so the halves are done first: O(n^3).
// Bounds: n: 1..=400; a: 1..=1e9
let n: 1..=400 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let size = prefix(a)

var cost: [I.ranges]Int = 0
for r in I.ranges if r.len >= 2:
    cost[r] = r.splits.map((x, y) => cost[x] + cost[y]).min()! + size.fold(r)
print(cost[I.range])

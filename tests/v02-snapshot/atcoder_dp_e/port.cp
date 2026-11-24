// Source: AtCoder Educational DP E (Knapsack 2), https://atcoder.jp/contests/dp/tasks/dp_e
// Problem: n <= 100 items with weight w_i <= W <= 1e9 and value v_i <= 1000. Print the largest
//   total value of a subset with total weight <= W.
// Solution: weights are huge but total value <= 1e5, so index by value: lightest[t] is the least
//   weight reaching value exactly t (None if impossible), a 0/1 knapsack downward over t. Print
//   the largest t with lightest[t] <= W. O(n * sum v).
let (n, cap): (1..=100, 1..=1e9) = read()
let items: [0..n](w: 1..=1e9, v: 1..=1000) = read()

let top = sum(v for (_, v) in items)
var lightest: [0..=top]?Int = None
lightest[0] = 0
for (w, v) in items:
    for total in (v..=top).rev: lightest[total] min= lightest[total - v] + w
print([total for total in 0..=top if lightest[total] != None and lightest[total]! <= cap].max()!)

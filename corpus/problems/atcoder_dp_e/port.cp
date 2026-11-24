// Source: AtCoder Educational DP E (Knapsack 2), https://atcoder.jp/contests/dp/tasks/dp_e
// limits: 2000 ms, 1024 MB
// Problem: n <= 100 items with weight w_i <= W <= 1e9 and value v_i <= 1000. Print the largest
//   total value of a subset with total weight <= W.
// Solution: weights are huge but the total value is at most 100 * 1000 = 1e5, so index by
//   value: lightest[t] is the least weight reaching value exactly t (None if no subset does),
//   a 0/1 knapsack going down over t. Print the largest t with lightest[t] <= W. O(n * sum v).
// Bounds: n: 1..=100; cap, items.w: 1..=1e9; items.v: 1..=1000
let (n, cap): (1..=100, 1..=1e9) = read()
let items: [n](w: 1..=1e9, v: 1..=1000) = read()

domain Value = 0..=sum(v for (_, v) in items)
var lightest: [Value]?Int = None
lightest[Value.first!] = 0
for (w, v) in items:
    for t in Value.rev:
        if let rest = Value.at(t - v): lightest[t] min= lightest[rest] + w
print(Value.all.filter(t => lightest[t].is_some_and(x => x <= cap)).max()!)

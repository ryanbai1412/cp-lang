// Source: EGOI 2023 Day 1 P1 (Inflation), https://dmoj.ca/problem/egoi23p1
// limits: 3000 ms, 1024 MB
// Problem: N <= 3e5 prices p_i <= 1e6, then Q <= 1e5 days, each "INFLATION x" (add x to every
//   price) or "SET x y" (every price equal to x becomes y), x, y <= 1e6. After each day print
//   the sum of all prices (needs 64 bits).
// Solution: keep the inflation so far and how many prices sit at each price less it; the sum
//   is the base sum plus N * inflation. SET moves the whole count at x - inflation onto
//   y - inflation and adds (y - x) * count to the base sum. O((N + Q) log N).
// Bounds: n: 1..=3e5; prices: 0..=1e6; q: 1..=1e5; x, y: 0..=1e6
// original: corpus/submissions/5678049.cpp

let n: 1..=3e5 = read()
let prices: [n]0..=1e6 = read()
var at = Counter(prices)  // count per price less the inflation
var base = prices.sum()
var inflation = 0

let q: 1..=1e5 = read()
for _ in 0..q:
    let day: Str = read()
    if day == "INFLATION":
        let x: 0..=1e6 = read()
        inflation += x
    else:
        let (x, y): (0..=1e6, 0..=1e6) = read()
        if x != y:
            if let moved = at.remove(x - inflation):
                at.add(y - inflation, moved)
                base += (y - x) * moved
    print(base + n * inflation)

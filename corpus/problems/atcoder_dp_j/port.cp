// Source: AtCoder Educational DP Contest J, Sushi, https://atcoder.jp/contests/dp/tasks/dp_j
// Problem: N <= 300 dishes hold 1 to 3 pieces each. Each step rolls a uniform dish and eats a
//   piece from it if there is one. Print the expected number of steps until all are eaten
//   (relative error 1e-9).
// Solution: only the counts (a, b, c) of dishes with 1, 2 and 3 pieces matter. A state is left
//   after n / (a + b + c) steps on average, to (a-1, b, c), (a+1, b-1, c) or (a, b+1, c-1)
//   with chances a : b : c. Filling c, then b, then a ascending computes those first;
//   O(N^3) states.
// original: corpus/submissions/4131649.cpp
use real.float.{fixed}

let n: Int = read()
domain Pieces = 1..=3
var dishes: [Pieces]Int = 0
for _ in 0..n:
    let pieces: Pieces = read()
    dishes[pieces] += 1

domain Count = 0..=n
var steps: [Count, Count, Count]F64 = 0.0
for c in Count:
    for b in Count:
        for a in Count:
            let left = a.label + b.label + c.label
            if left == 0 or left > n: continue
            var after = F64(n)
            if let a1 = a.prev: after += a.label * steps[a1, b, c]
            if let b1 = b.prev: after += b.label * steps[a.next!, b1, c]
            if let c1 = c.prev: after += c.label * steps[a, b.next!, c1]
            steps[a, b, c] = after / left
print(steps[Count(dishes[Pieces(1)]), Count(dishes[Pieces(2)]), Count(dishes[Pieces(3)])], precision: 9)

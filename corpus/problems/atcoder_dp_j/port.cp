// Educational DP Contest J, Sushi: n dishes hold 1, 2 or 3 pieces of sushi.
// Each step a dish is picked at random and, if it is not empty, one piece
// is eaten from it. Print the expected number of steps until all is eaten.
//
// Only the number of dishes with 1, 2 and 3 pieces matters. From (a, b, c)
// a step hits a non-empty dish with chance (a + b + c) / n, so it takes
// n / (a + b + c) steps on average to leave the state, and it leaves to
// (a-1, b, c), (a+1, b-1, c) or (a, b+1, c-1) with chances a : b : c.
// Filling by c, then b, then a visits those three first.
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

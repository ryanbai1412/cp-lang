// Source: Yet Another Contest 6 P5 (No More Coyotes), https://dmoj.ca/problem/yac6p5
// Problem: 2 <= N <= 1e6 sheep at (X, Y) (|X|, |Y| <= 1e9), each seeing one quadrant V in 0..=3
//   (0: x >= and y >= its own, 1: x >=, y <=, 2: x <=, y <=, 3: x <=, y >=). A move is a unit
//   step or a 90 degree turn (V +- 1 mod 4). Print the fewest moves so every sheep sees all.
// Solution: V is a pair of bits, one per axis, and a turn flips one of them, so the axes are
//   independent. On one axis the sheep looking up must share the minimum and the ones looking
//   down the maximum: sort, try every split into a low and a high group, gather each at its
//   median (prefix sums) and pay a turn per sheep in the low group looking down or in the high
//   group looking up. Everyone at one spot needs no turns. Sum the two axes; O(N log N).

let n: 2..=1e6 = read()
struct Sheep:
    x: -1e9..=1e9
    y: -1e9..=1e9
    v: 0..=3
let flock: [n]Sheep = read()

// One axis: each sheep's coordinate and whether it looks down (toward smaller values).
fn axis(sheep: List<(Int, Bool)>) -> Int:
    let at = sheep.sorted()
    let pre = at.map(a => a.0).scan(+)
    // gathering sorted sheep l..r at their median
    fn gather(l: Int, r: Int) -> Int:
        let h = (r - l) / 2
        pre.fold(r - h..r) - pre.fold(l..l + h)
    var turns = at.count(a => not a.1)  // split before everyone: all must look down
    var best = gather(0, n)
    for i in 0..=n:
        best min= gather(0, i) + gather(i, n) + turns
        if i < n: turns += if at[i].1: 1 else: -1
    best

print(axis([(s.x, s.v >= 2) for s in flock]) + axis([(s.y, s.v == 1 or s.v == 2) for s in flock]))

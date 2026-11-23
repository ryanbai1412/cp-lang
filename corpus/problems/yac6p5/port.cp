// Source: Yet Another Contest 6 P5 (No More Coyotes), https://dmoj.ca/problem/yac6p5
// Problem: N <= 1e6 sheep at (X, Y) (|X|, |Y| <= 1e9), each seeing one quadrant V in 0..=3
//   (0: x >= and y >= its own, 1: x >=, y <=, 2: x <=, y <=, 3: x <=, y >=). A move is a unit
//   step or a 90 degree turn (V +- 1 mod 4). Print the fewest moves so every sheep sees all.
// Solution: V is a pair of bits, one per axis, and a turn flips one of them, so the axes are
//   independent. On one axis the sheep seeing larger coordinates must share the minimum and
//   the others the maximum: sort, try every split into a low and a high group, gather each at
//   its median (prefix sums) and pay a turn per sheep facing the wrong way (cadd). Sum the two
//   axes; O(N log N).

let n: 1..=200_000 = read()
var xs: List<(Int, Int)> = []
var ys: List<(Int, Int)> = []
for i in 0..n:
    let x: -1_000_000_000..=1_000_000_000 = read()
    let y: -1_000_000_000..=1_000_000_000 = read()
    let v: 0..=3 = read()
    var bx = 1
    if v == 0 or v == 1:
        bx = 0
    var by = 1
    if v == 0 or v == 3:
        by = 0
    xs.push((x, bx))
    ys.push((y, by))

var pre: [0..=n]Int = 0

// cost of gathering sorted positions l..r at their median
fn cost(l: Int, r: Int) -> Int:
    let h = (r - l) / 2
    (pre[r] - pre[r - h]) - (pre[l + h] - pre[l])

fn solve(mut arr: List<(Int, Int)>) -> Int:
    arr.sort()
    var cadd = 0
    for i in 0..n:
        pre[i + 1] = pre[i] + arr[i].0
        if arr[i].1 == 0:
            cadd += 1
    var best = cost(0, n)
    for i in 0..=n:
        let cur = cost(0, i) + cost(i, n) + cadd
        if cur < best:
            best = cur
        if i < n:
            if arr[i].1 == 0:
                cadd -= 1
            else:
                cadd += 1
    best

print(solve(mut xs) + solve(mut ys))

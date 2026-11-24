// Source: RPENG 2, which is IOI 1998 Polygon, https://dmoj.ca/problem/ioi98p6
// Problem: a polygon of N <= 50 integer vertices whose edges are + ('t') or * ('x'). Remove one
//   edge, then repeatedly merge two neighbours by their edge's operation. Print the best final
//   value, then every edge (increasing) whose removal can reach it.
// Solution: removing edge i leaves a chain, and a chain's best value comes from splitting it at
//   its last operation. Products of negatives need both ends, so lo/hi[i, len] keep the least
//   and largest value of the chain of len + 1 vertices from vertex i round the ring: + combines
//   lo with lo and hi with hi, * all four products. O(N^3).
// original: corpus/submissions/4810287.cpp
// Bounds: n: 1..=50
let n: 1..=50 = read()
var times: List<Bool> = []
var val: List<Int> = []
for _ in 0..n:
    let op: Char = read()
    times.push(op == 'x')
    let x: Int = read()
    val.push(x)

// lo/hi[i, len]: range of results for the chain of len + 1 vertices starting
// at vertex i, walking clockwise around the ring; edge j joins j - 1 and j
var lo: [0..n, 0..n]Int = 0
var hi: [0..n, 0..n]Int = 0
for i in 0..n:
    lo[i, 0] = val[i]
    hi[i, 0] = val[i]
for len in 1..n:
    for i in 0..n:
        var least = 1e18
        var most = -1e18
        for left in 0..len:
            let j = (i + left + 1) % n
            let right = len - left - 1
            if times[j]:
                for x in [lo[i, left], hi[i, left]]:
                    for y in [lo[j, right], hi[j, right]]:
                        least min= x * y
                        most max= x * y
            else:
                least min= lo[i, left] + lo[j, right]
                most max= hi[i, left] + hi[j, right]
        lo[i, len] = least
        hi[i, len] = most

var best = -1e18
for i in 0..n: best max= hi[i, n - 1]
print(best)
var cuts: List<Int> = []
for i in 0..n:
    if hi[i, n - 1] == best: cuts.push(i + 1)
print(cuts)

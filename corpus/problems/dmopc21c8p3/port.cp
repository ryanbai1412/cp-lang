// Source: DMOPC '21 Contest 8 P3 (Weaker Data), https://dmoj.ca/problem/dmopc21c8p3
// Problem: print the lexicographically smallest permutation of 1..N with exactly K valleys,
//   triples i < j < k with P_i > P_j < P_k, or -1. N <= 1e6, K <= 1e18.
// Solution: place values from the smallest up: the smallest unplaced value is the middle of
//   l * r valleys, l and r the free positions left and right of it. ways[i], the most valleys
//   i values make (l, r split evenly each time), is capped at K + 1. A value takes the first
//   free position while the rest can still make the remaining count, else the middle one (offset
//   1 in the lone case of 5 left and 4 needed). A Fenwick tree over free positions finds the
//   chosen one. O(N log N).
// original: corpus/submissions/4537755.cpp

let n: 1..=1e6 = read()
let k: 0..=1e18 = read()
domain Pos = 0..n
domain Val = 1..=n

// most[i]: the most valleys i values make, capped past k
var most: [0..=n]Int = 0
for i in 1..=n:
    let l = i / 2
    most[i] = min(k + 1, most[i - 1] + l * (i - l - 1))

if k > most[n]:
    print(-1)
else:
    // offset[v]: among the free positions, the one value v takes, counted from the left
    var offset: [Val]Int = 0
    var left = k
    for v in Val:
        let free = n - v.ord
        if left <= most[free - 1]:
            offset[v] = 0
        elif free == 5 and left == 4:
            offset[v] = 1
            left -= 3
        else:
            let l = (free - 1) / 2
            offset[v] = l
            left -= l * (free - l - 1)

    var open = Fenwick<Pos, Int>([Pos] _ => 1)
    var perm: [Pos]Val = Val.first!
    for v in Val:
        let p = open.first_gt(offset[v])!.left!
        perm[p] = v
        open.add(p, -1)
    print(perm)

// Source: DMOPC '21 Contest 8 P3 (Weaker Data), https://dmoj.ca/problem/dmopc21c8p3
// Problem: print the lexicographically smallest permutation of 1..N with exactly K valleys,
//   triples i < j < k with P_i > P_j < P_k, or -1. N <= 1e6, K <= 1e18.
// Solution: place values from the smallest up: the smallest unplaced value is the middle of
//   l * r valleys, l and r the free positions left and right of it. ways[i], the most valleys
//   i values make (l, r split evenly each time), is capped at K + 1. A value takes the first
//   free position while the rest can still make the remaining count, else the middle one (offset
//   1 in the lone case of 5 left and 4 needed). A Fenwick tree over free positions finds the
//   chosen one. O(N log N).
// Runnable port (proto/cp.py lib-run) of dmopc21c8p3.cp
// original: corpus/submissions/4537755.cpp

let (n, k): (Int, Int) = read()
domain Pos = 0..n
domain Val = 1..=n

let cap = k + 1
var ways: List<Int> = [0]
for i in 1..=n:
    let l = i / 2
    let r = i - l - 1
    let prev = ways[i - 1]
    let add = min(cap - prev, l * r)
    ways.push(prev + add)

if k > ways[n]:
    print(-1)
else:
    var offsets: [Val]Int = [Val] _ => 0
    var rem = k
    for v in Val:
        let tot = n - v.to_idx
        if rem <= ways[tot - 1]:
            offsets[v] = 0
        elif tot == 5 and rem == 4:
            offsets[v] = 1
            rem -= 3
        else:
            let l = (tot - 1) / 2
            offsets[v] = l
            rem -= l * (tot - l - 1)

    // `Set.nth(i)` is not yet on `proto/cpi`; use `Fenwick.lower_bound` on 1s.
    var bit = Fenwick<Pos, Int>([Pos] _ => 1)
    var perm: [Pos]Int = [Pos] _ => 0
    for v in Val:
        let b = bit.first_ge(offsets[v] + 1)!
        let p = b.left!
        perm[p] = v.to_idx + 1
        bit.add(p, -1)

    print(perm, sep: " ")

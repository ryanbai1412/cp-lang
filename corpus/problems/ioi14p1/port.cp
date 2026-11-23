// Source: IOI 2014 P1 (Rail), https://dmoj.ca/problem/ioi14p1
// Problem: interactive (function task). n stations sit in distinct blocks (at most 1e6) of a
//   rail line, each type C or D; station 0 is type C at block `first`. getDistance(i, j) gives
//   the shortest route length; recover every station's block and type within each subtask's
//   limit on getDistance calls.
// Solution: query 0 against every station and sort by distance; the nearest is the D just
//   right of station 0. In that order, keep the leftmost C (l) and rightmost D (r): two
//   queries give the block if the station is a D right of l (loc[l] + dl) or a C left of r
//   (loc[r] - dr), and the turning block between them decides: a placed station there by its
//   type, else D iff it lies right of the rightmost C so far. 3n - 5 queries; placed stations
//   are looked up by linear scan, O(n^2) time.
// Stdin: n, then hidden loc[0..n] and kind[0..n] (1 or 2). Simulates getDistance(i, j)
// internally, runs findLocation, and prints recovered loc[i] and kind[i].
// original: corpus/submissions/4691718.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn main_solve():
    let n: Int = read()
    var true_loc: List<Int> = []
    var true_kind: List<Int> = []
    for _ in 0..n: true_loc.push(ri())
    for _ in 0..n: true_kind.push(ri())

    fn get_distance(i: Int, j: Int) -> Int:
        if i == j: return 0
        let a = true_loc[i]
        let b = true_loc[j]
        let ka = true_kind[i]
        let kb = true_kind[j]
        if ka == 1 and kb == 2 and a < b: return b - a
        if ka == 2 and kb == 1 and a > b: return a - b
        if ka == 1 and kb == 1:
            var best = 1_000_000_000
            for k in 0..n:
                if true_kind[k] == 2 and true_loc[k] > max(a, b):
                    best min= (true_loc[k] - a) + (true_loc[k] - b)
            return best
        if ka == 2 and kb == 2:
            var best = 1_000_000_000
            for k in 0..n:
                if true_kind[k] == 1 and true_loc[k] < min(a, b):
                    best min= (a - true_loc[k]) + (b - true_loc[k])
            return best
        if ka == 1 and kb == 2:
            var rmin = 1_000_000_000
            for k in 0..n:
                if true_kind[k] == 2 and true_loc[k] > a: rmin min= true_loc[k]
            var lmax = -1_000_000_000
            for k in 0..n:
                if true_kind[k] == 1 and true_loc[k] < b: lmax max= true_loc[k]
            return (rmin - a) + (rmin - lmax) + (b - lmax)
        var lmax = -1_000_000_000
        for k in 0..n:
            if true_kind[k] == 1 and true_loc[k] < a: lmax max= true_loc[k]
        var rmin = 1_000_000_000
        for k in 0..n:
            if true_kind[k] == 2 and true_loc[k] > b: rmin min= true_loc[k]
        return (a - lmax) + (rmin - lmax) + (rmin - b)

    var dist = [0..n] _ => 0
    var ord: List<(Int, Int)> = []
    for i in 1..n:
        dist[i] = get_distance(0, i)
        ord.push((dist[i], i))
    ord.sort()

    var loc = [0..n] _ => 0
    var kind = [0..n] _ => 0
    loc[0] = true_loc[0]
    kind[0] = 1

    if n > 1:
        let s1 = ord[0].1
        var l = 0
        var r = s1
        var open_r = loc[0]
        loc[s1] = true_loc[0] + dist[s1]
        kind[s1] = 2
        var placed_x: List<Int> = [loc[0], loc[s1]]
        var placed_k: List<Int> = [1, 2]

        fn lookup(x: Int) -> Int:
            for idx in 0..placed_x.len:
                if placed_x[idx] == x: return placed_k[idx]
            return 0

        for idx in 1..ord.len:
            let a = ord[idx].1
            let dl = get_distance(l, a)
            let dr = get_distance(r, a)
            let posl = loc[l] + dl
            let posr = loc[r] - dr
            let mid = (loc[r] + posl - dr) / 2
            let mk = lookup(mid)
            let cl = if mk != 0: mk == 1 else: mid > open_r
            if cl:
                loc[a] = posl
                kind[a] = 2
                if loc[a] > loc[r]: r = a
            else:
                loc[a] = posr
                kind[a] = 1
                if loc[a] < loc[l]: l = a
                open_r max= loc[a]
            placed_x.push(loc[a])
            placed_k.push(kind[a])

    for i in 0..n: print(loc[i], kind[i])

main_solve()

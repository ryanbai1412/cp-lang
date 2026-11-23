// Source: SDOI 2009, School Canteen, https://dmoj.ca/problem/etsr23r2p4
// Problem: n <= 1000 students queue; student i has taste T_i <= 1000 and tolerance B_i <= 7:
//   at most the B_i people right behind i may be served before i. Cooking a dish after one of
//   taste a costs a xor b (the first is free). Print the least total time. C <= 5 tests.
// Solution: everyone before the first unserved student ci is served, and nobody beyond ci + 7
//   can be, so a state is (ci, last served student, bitmask of served students in the window
//   after ci). The next student is any unserved pos up to the least pos + B_pos over unserved
//   pos in the window; memoised recursion over a Map: O(n * 2^8 * 16 * 8) per test.
// original: corpus/submissions/5494510.cpp

let tcs: Int = read()
for _ in 0..tcs:
    let n: Int = read()
    domain Item = 0..n
    var val: [Item]Int = [Item] _ => 0
    var tol: [Item]Int = [Item] _ => 0
    for i in Item:
        let (a, b): (Int, Int) = read()
        val[i] = a
        tol[i] = b

    var memo = Map<(Int, Int, Int), Int>()

    fn dp(ind: Int, prv: Int, mask: Int) -> Int:
        var ci = ind
        var cm = mask
        while ci < n and (cm & 1) == 1:
            ci += 1
            cm = cm >> 1
        if ci >= n: return 0

        var mxloc = n - 1
        for k in 0..=14:
            let pos = ci + k
            if pos >= n: break
            if ((cm >> k) & 1) == 0:
                let lim = pos + tol[Item(pos)]
                if lim < mxloc: mxloc = lim

        let key = (ci, prv, cm)
        if let v = memo.get(key): return v

        var best: ?Int = None
        for pos in ci..=mxloc:
            let bit = pos - ci
            if ((cm >> bit) & 1) == 1: continue
            let cost = if prv == -1: 0 else: val[Item(prv)] ^ val[Item(pos)]
            let cand = cost + dp(ci, pos, cm | (1 << bit))
            if best == None or cand < best!:
                best = cand
        memo[key] = best!
        best!

    print(dp(0, -1, 0))

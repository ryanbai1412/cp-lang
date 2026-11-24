// Source: Codeforces 2101 D, Mani and Segments, https://codeforces.com/contest/2101/problem/D
// limits: 3000 ms, 250 MB
// Problem: multitest (sum n <= 2e5): a permutation a of length n. Count the non-empty
//   subarrays b that are cute: LIS(b) + LDS(b) = |b| + 1.
// Solution: b is cute iff no element lies strictly between two earlier elements in value,
//   and none strictly between two later ones. Monotone stacks give, for each i, the nearest
//   earlier smaller and larger elements; any start at or before both breaks at i, and a
//   suffix minimum turns that into bad_r[l], the first end that breaks start l (the reversed
//   scan gives bad_l). Cute subarrays are the union of rectangles
//   [bad_l[i]+1..=i] x [i..bad_r[i]]; subtracting adjacent overlaps counts each once. O(n).


@!multitest
let n: 1..=200_000 = read()
domain Pos = 0..n
let a: [Pos]1..=n = read()

fn first_bad(xs: [Pos]Int) -> [Pos]Int:
    var cur: [Pos]Int = n
    var lo: List<(val: Int, pos: Int)> = []
    var hi: List<(val: Int, pos: Int)> = []
    for i in Pos:
        let v = xs[i]
        while let top = lo.last and v < top.val: lo.pop()
        while let top = hi.last and v > top.val: hi.pop()
        if let x = lo.last and let y = hi.last:
            cur[Pos(min(x.pos, y.pos))] min= i.ord
        lo.push((v, i.ord))
        hi.push((v, i.ord))
    let suf = cur.rev.scan(min)
    [Pos] i => suf[i.l]

let bad_r = first_bad([Pos] i => a[i])
let bad_l_rev = first_bad([Pos] i => a[Pos(n - 1 - i.ord)])
let bad_l = [Pos] i => n - 1 - bad_l_rev[Pos(n - 1 - i.ord)]

let total = sum((bad_r[i] - i.ord) * (i.ord - bad_l[i]) for i in Pos)
let overlap = sum((bad_r[k] - (k.ord + 1)) * (k.ord - bad_l[k.next!]) for k in Pos if k < Pos.last!)
print(total - overlap)

// Runnable port (proto/cp.py lib-run) of statement-first cco21p1.cp
// original: corpus/submissions/4538969.cpp

let (n, k, q): (Int, Int, Int) = read()
domain Pos = 1..=n
domain Val = 1..=k
domain Query = 0..q
let a: [Pos]Val = read()
let swaps: [Query]Int = read()

var occ: [Val]List<Int> = [Val] _ => []
for i in Pos: occ[a[i]].push(i.to_idx)

fn count_before(xs: List<Int>, ys: List<Int>) -> Int:
    if xs.len <= ys.len:
        var cnt = 0
        for p in xs:
            let lo = ys.first_ge(p) ?? ys.len
            cnt += ys.len - lo
        cnt
    else:
        xs.len * ys.len - count_before(ys, xs)

var bit = Fenwick<Val, Int>.new()
var inv = 0
for i in Pos:
    if a[i] < Val.last!:
        inv += bit.fold(a[i].r..Val.last!.r)
    bit.add(a[i], 1)

var order = [Val] v => v
var cache = Map<Int, Int>()
for qi in Query:
    let j = Val(swaps[qi])
    let j1 = Val(swaps[qi] + 1)
    let (x, y) = (order[j], order[j1])
    let (lo, hi) = if x.to_idx < y.to_idx: (x, y) else: (y, x)
    let key = lo.to_idx * k + hi.to_idx
    var before_lo_hi = 0
    if key in cache:
        before_lo_hi = cache[key]
    else:
        before_lo_hi = count_before(occ[lo], occ[hi])
        cache[key] = before_lo_hi
    let before_xy = if x.to_idx < y.to_idx: before_lo_hi else: occ[x].len * occ[y].len - before_lo_hi
    inv += 2 * before_xy - occ[x].len * occ[y].len
    order.swap(j, j1)
    print(inv)

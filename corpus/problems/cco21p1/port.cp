// Source: CCO 2021 P1, Swap Swap Sort, https://dmoj.ca/problem/cco21p1
// Problem: an array of n <= 1e5 values in 1..k and a target order of 1..k (initially 1..k).
//   Each of q <= 1e6 queries swaps the target order's entries j and j + 1; after each, print
//   the least number of adjacent swaps that sort the array by the current target order.
// Solution: the answer is the inversion count under the target order; swapping adjacent values
//   x, y in the order only flips the pairs made of one x and one y. With b(x, y) the pairs with
//   an x before a y, the count changes by 2 b(x, y) - |x| |y|. b comes from the positions
//   lists, binary-searching each element of the shorter list in the longer one, memoised per
//   pair in a Map. Initial count by a Fenwick tree; O(n sqrt(q) log n) over distinct pairs.
// original: corpus/submissions/4538969.cpp

let (n, k, q): (Int, Int, Int) = read()
domain Pos = 1..=n
domain Val = 1..=k
domain Query = 0..q
let a: [Pos]Val = read()
let swaps: [Query]Int = read()

var occ: [Val]List<Int> = [Val] _ => []
for i in Pos: occ[a[i]].push(i.ord)

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
    let (lo, hi) = if x.ord < y.ord: (x, y) else: (y, x)
    let key = lo.ord * k + hi.ord
    var before_lo_hi = 0
    if let val = cache.get(key):
        before_lo_hi = val
    else:
        before_lo_hi = count_before(occ[lo], occ[hi])
        cache[key] = before_lo_hi
    let before_xy = if x.ord < y.ord: before_lo_hi else: occ[x].len * occ[y].len - before_lo_hi
    inv += 2 * before_xy - occ[x].len * occ[y].len
    order.swap(j, j1)
    print(inv)

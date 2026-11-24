// Source: CCO 2021 P1, Swap Swap Sort, https://dmoj.ca/problem/cco21p1
// Problem: an array of n <= 1e5 values in 1..k and a target order of 1..k (initially 1..k).
//   Each of q <= 1e6 queries swaps the target order's entries j and j + 1; after each, print
//   the least number of adjacent swaps that sort the array by the current target order.
// Solution: the answer is the inversion count under the target order; swapping adjacent values
//   x, y in the order only flips the pairs made of one x and one y. With b(x, y) the pairs with
//   an x before a y, the count changes by 2 b(x, y) - |x| |y|. b comes from each value's
//   positions (std `buckets`), binary-searching each position of the shorter list in the
//   longer one, memoised per pair. Initial count by std `inversions`. O(n sqrt(q) log n).
// original: corpus/submissions/4538969.cpp
// Bounds: n: 1..=1e5; k: 2..=1e5; q: 1..=1e6; a: 1..=k; swaps: 1..k
let (n, k, q): (1..=1e5, 2..=1e5, 1..=1e6) = read()
domain Pos = 1..=n
domain Val = 1..=k
domain Slot = 1..k  // swap entries j and j + 1 of the order
domain Query = 0..q
let a: [Pos]Val = read()
let swaps: [Query]Slot = read()

let at = a.buckets()  // each value's positions, ascending
var known = Map<(Val, Val), Int>()

// pairs with an x before a y
fn before(x: Val, y: Val) -> Int:
    if x > y: return at[x].len * at[y].len - before(y, x)
    if let b = known.get((x, y)): return b
    let (xs, ys) = (at[x], at[y])
    let b = if xs.len <= ys.len: sum(ys.len - (ys.first_gt(p) ?? ys.len) for p in xs)
        else: sum(xs.first_gt(p) ?? xs.len for p in ys)
    known[(x, y)] = b
    b

var inv = a.inversions()
var order = [Val] v => v
for s in swaps:
    let j = Val(Int(s))
    let (x, y) = (order[j], order[j.next!])
    inv += 2 * before(x, y) - at[x].len * at[y].len
    order.swap(j, j.next!)
    print(inv)

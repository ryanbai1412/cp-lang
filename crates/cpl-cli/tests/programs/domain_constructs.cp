use std.bitset.{Bitset}

enum Sign: neg = -1, zero = 0, pos = 1

let n: 1..=10 = read()
domain V = 1..=n
let edges: List<(u: V, v: V)> = [(V(1), V(2)), (V(2), V(3)), (V(3), V(4))]
let adj = adjacency(edges, into: Bitset)

// 1. Domain-wide subsets: V.all, Bitset<V>, for v in s, without, -, find, any, len
let full: Bitset<V> = V.all
let without2 = V.all.without(V(2))
let diff = V.all - adj[V(2)]
var sum_diff = 0
for v in diff: sum_diff += Int(v)
let first_big = diff.find(v => Int(v) >= 2)
let has_four = without2.any(v => Int(v) == 4)
print(full.len, without2.len, diff.len, sum_diff, first_big!, has_four)

// 2. Enum-indexed arrays + comprehensions + explicit values
let vals: [Sign]Int = [Sign] s => Int(s) * 10
var total_sign = 0
for s in Sign: total_sign += vals[s]
print(vals[Sign.neg], vals[Sign.zero], vals[Sign.pos], total_sign)

// 3. Domain-level sequence ops: partition, sort(by:), group(by:), at, first(p)
let (evens, odds) = V.partition(v => Int(v) % 2 == 0)
let weight: [V]Int = [V] v => (Int(v) * 7) % 5
let by_w = V.sort(by: weight)
let groups = V.group(by: v => Int(v) % 2)
let at2 = V.at(2)
let gt2 = V.first(v => Int(v) > 2)
print(evens.len, odds.len, by_w, groups.len, at2, gt2!)

// 4. Item/Cut pairs + minmax
domain Item = 1..=n
domain Cut = 0..=n
let p: [Item]Cut = [Item] i => Cut(Int(i) - 1)
let s: [Item]Cut = [Item] i => Cut(n - Int(i) + 1)
let (lo, hi) = minmax(p[Item(1)], s[Item(1)])
let r = lo..hi
print(lo, hi, r.len, choose2(n + 1))

// 5. D.bounds directly in SegTree, LazySegTree, Fenwick
var st = SegTree<V.bounds, Int>(+, fill: 0)
var lst = LazySegTree<V.bounds, Int>(AddMax, fill: 0)
var fw = Fenwick<V.bounds, Int>()
let v2 = V(2)
st.set(v2.l, 2)
st.set(v2.r, 5)
lst.set(v2.r, 7)
lst.apply(..=v2.l, 1)
fw.add(v2.l, 4)
fw.add(v2.r, 3)
print(st.fold(..=v2.r), lst.all, lst.fold(..=v2.l), fw.sum(..=v2.r))

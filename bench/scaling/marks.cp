use std.marks.{Marks, MarkMap, Interner, SparseSet, Arena}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0

for _ in 0..1500:
    let m = Marks<V>() // timed: marks.Marks.new
    if m.is_empty: s += 1
var m = Marks<V>()
for i in 0..reps:
    if m.add(V(i * 7919 % n)): s += 1 // timed: marks.Marks.add
    if m.contains(V(i * 13 % n)): s += 1 // timed: marks.Marks.contains
    if m.remove(V(i * 31 % n)): s += 1 // timed: marks.Marks.remove
    s += m.len // timed: marks.Marks.len
    if m.is_empty: s += 1 // timed: marks.Marks.is_empty
for i in 0..n: m.add(V(i))
for i in 0..reps:
    m.add(V(i % n))
    m.clear() // timed: marks.Marks.clear

for _ in 0..1500:
    let mm = MarkMap<V, Int>(7) // timed: marks.MarkMap.new
    if mm.is_empty: s += 1
var mm = MarkMap<V, Int>(0)
for i in 0..reps:
    mm.set(V(i * 7919 % n), i) // timed: marks.MarkMap.set
    if mm.contains(V(i * 13 % n)): s += 1 // timed: marks.MarkMap.contains
    s += mm.get(V(i * 17 % n)) ?? 0 // timed: marks.MarkMap.get
    s += mm.remove(V(i * 31 % n)) ?? 0 // timed: marks.MarkMap.remove
    s += mm.len // timed: marks.MarkMap.len
    if mm.is_empty: s += 1 // timed: marks.MarkMap.is_empty
for i in 0..n: mm.set(V(i), i)
for i in 0..reps:
    mm.set(V(i % n), i)
    mm.clear() // timed: marks.MarkMap.clear

for i in 0..reps:
    let t = Interner<Int>() // timed: marks.Interner.new
    if t.is_empty: s += 1

for _ in 0..1500:
    let ss = SparseSet<V>() // timed: marks.SparseSet.new
    if ss.is_empty: s += 1
var ss = SparseSet<V>()
for i in 0..reps:
    if ss.add(V(i * 7919 % n)): s += 1 // timed: marks.SparseSet.add
    if ss.contains(V(i * 13 % n)): s += 1 // timed: marks.SparseSet.contains
    if ss.remove(V(i * 31 % n)): s += 1 // timed: marks.SparseSet.remove
    s += ss.len // timed: marks.SparseSet.len
    if ss.is_empty: s += 1 // timed: marks.SparseSet.is_empty
for i in 0..n: ss.add(V(i))
for _ in 0..200:
    for x in ss.each(): // timed: marks.SparseSet.each
        s += Int(x) // within: marks.SparseSet.each
for i in 0..reps:
    ss.add(V(i % n))
    ss.clear() // timed: marks.SparseSet.clear

for i in 0..reps:
    let a = Arena<Int>(cap: n) // timed: marks.Arena.new
    if a.is_empty: s += 1
var a = Arena<Int>()
for i in 0..reps:
    let id = a.alloc(i) // timed: marks.Arena.alloc
    s += a.get(id) // timed: marks.Arena.get
    a.set(id, i + 1) // timed: marks.Arena.set
    s += a.len // timed: marks.Arena.len
    if a.is_empty: s += 1 // timed: marks.Arena.is_empty
for i in 0..reps:
    a.alloc(i)
    a.clear() // timed: marks.Arena.clear
var ab = Arena<(Int, Int)>()
for i in 0..n: ab.alloc((i, i))
print(s)

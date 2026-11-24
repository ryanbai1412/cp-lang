let n: Int = read()
let ys = [i for i in 0..n]
var base: Set<Int> = Set()
var other: Set<Int> = Set()
var mp: Map<Int, Int> = Map()
for i in 0..n:
    base.insert(i * 2)
    other.insert(i * 3)
    mp[i] = i
let inc = (v: Int) => v + 1 // within: seq.update
let neg = (x: Int) => x < 0 // within: seq.pop_if
var s = 0
var l = ys.clone()
for r in 0..262144:
    let k = r % n
    s += mp.get_or_insert(k, 0) // timed: seq.get_or_insert
    mp.update(k, 0, inc) // timed: seq.update
    l.swap_at(k, r * 7 % n) // timed: seq.swap_at
    s += l.pop_if(neg) ?? 0 // timed: seq.pop_if
    var e = [r]
    s += e.take_all().len // timed: seq.take_all
for r in 0..16:
    var a = base.clone()
    a.union(other) // timed: seq.union
    var b = base.clone()
    b.intersect(other) // timed: seq.intersect
    var c = base.clone()
    c.subtract(other) // timed: seq.subtract
    s += a.len + b.len + c.len
print(s)

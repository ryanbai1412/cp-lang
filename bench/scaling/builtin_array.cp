// The built-in [n]Int: O(1) access, O(log n) searches of a sorted one, O(n) passes.
let n: Int = read()
var a: [n]Int = [i * 7919 % n for i in 0..n]
let b: [n]Int = [i for i in 0..n]
var s = 0
for i in 0..64 * n:
    let k = i * 13 % n
    s += a.get(k) ?? 0 // timed: builtin.Array.get
    a.swap(k, (k + 1) % n) // timed: builtin.Array.swap
    s += a.cut_after(k) // timed: builtin.Array.cut_after
    s += a.cut_before(k) // timed: builtin.Array.cut_before
for i in 0..4 * n:
    let k = i * 13 % n
    s += b.first_ge(k) ?? 0 // timed: builtin.Array.first_ge
    s += b.first_gt(k) ?? 0 // timed: builtin.Array.first_gt
    s += b.last_le(k) ?? 0 // timed: builtin.Array.last_le
    s += b.last_lt(k) ?? 0 // timed: builtin.Array.last_lt
let small = (x: Int) => x < n // within: builtin.Array.all
let big = (x: Int) => x >= n // within: builtin.Array.any
let twice = (x: Int) => 2 * x // within: builtin.Array.map
let odd = (x: Int) => x % 2 == 1 // within: builtin.Array.filter
let up = (p: Int, q: Int) => q - p < 64 // within: builtin.Array.reach
for r in 0..1024:
    var c = a.clone() // timed: builtin.Array.clone
    c.fill(r) // timed: builtin.Array.fill
    c.reverse() // timed: builtin.Array.reverse
    s += a.reversed().len // timed: builtin.Array.reversed
    s += a.map(twice).len // timed: builtin.Array.map
    s += a.filter(odd).count() // timed: builtin.Array.filter
    if a.contains(n): s += 1 // timed: builtin.Array.contains
    s += a.count(x => x == n) // timed: builtin.Array.count
    s += a.sum() // timed: builtin.Array.sum
    s += a.min() ?? 0 // timed: builtin.Array.min
    s += a.max() ?? 0 // timed: builtin.Array.max
    if a.all(small): s += 1 // timed: builtin.Array.all
    if a.any(big): s += 1 // timed: builtin.Array.any
    s += b.reach(while: up).len // timed: builtin.Array.reach
for r in 0..32:
    var c = a.clone()
    c.sort() // timed: builtin.Array.sort
    s += a.sorted().len // timed: builtin.Array.sorted
print(s)

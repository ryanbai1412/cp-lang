// The built-in List<Int> of n elements: O(1) access, O(log n) searches of a sorted one, O(n) passes.
let n: Int = read()
var xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
var s = 0
for i in 0..64 * n:
    let k = i * 13 % n
    xs.push(k) // timed: builtin.List.push
    s += xs.pop() ?? 0 // timed: builtin.List.pop
    s += xs.get(k) ?? 0 // timed: builtin.List.get
    xs.swap(k, (k + 1) % n) // timed: builtin.List.swap
    s += xs.cut_after(k) // timed: builtin.List.cut_after
    s += xs.cut_before(k) // timed: builtin.List.cut_before
    s += xs.first ?? 0 // timed: builtin.List.first
    s += xs.last ?? 0 // timed: builtin.List.last
    s += xs.push_id(k) // timed: builtin.List.push_id
    xs.pop()
for i in 0..4 * n:
    let k = i * 13 % n
    s += ys.first_ge(k) ?? 0 // timed: builtin.List.first_ge
    s += ys.first_gt(k) ?? 0 // timed: builtin.List.first_gt
    s += ys.last_le(k) ?? 0 // timed: builtin.List.last_le
    s += ys.last_lt(k) ?? 0 // timed: builtin.List.last_lt
let small = (x: Int) => x < n // within: builtin.List.all
let big = (x: Int) => x >= n // within: builtin.List.any
let twice = (x: Int) => 2 * x // within: builtin.List.map
let odd = (x: Int) => x % 2 == 1 // within: builtin.List.filter
let up = (p: Int, q: Int) => q - p < 64 // within: builtin.List.reach
// A read of a view runs in the reader's loop: `first` stops at the first element.
for i in 0..64 * n:
    s += xs.map(twice).first ?? 0 // timed: builtin.List.map
    s += xs.filter(odd).first ?? 0 // timed: builtin.List.filter
for r in 0..4096:
    xs.insert(0, r) // timed: builtin.List.insert
    s += xs.remove(0) // timed: builtin.List.remove
for r in 0..1024:
    var c = xs.clone()
    c.fill(r) // timed: builtin.List.fill
    s += c.to_list().len // timed: builtin.List.to_list
    c.reverse() // timed: builtin.List.reverse
    s += xs.reversed().len // timed: builtin.List.reversed
    if xs.contains(n): s += 1 // timed: builtin.List.contains
    s += xs.count(x => x == n) // timed: builtin.List.count
    s += xs.sum() // timed: builtin.List.sum
    s += xs.min() ?? 0 // timed: builtin.List.min
    s += xs.max() ?? 0 // timed: builtin.List.max
    if xs.all(small): s += 1 // timed: builtin.List.all
    if xs.any(big): s += 1 // timed: builtin.List.any
    s += xs.find(n) ?? 0 // timed: builtin.List.find
    s += ys.reach(while: up).len // timed: builtin.List.reach
for r in 0..32:
    var c = xs.clone()
    c.sort() // timed: builtin.List.sort
    s += xs.sorted().len // timed: builtin.List.sorted
    s += c[0]
print(s)

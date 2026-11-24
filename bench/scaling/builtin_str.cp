// The built-in Str of n chars: O(1) access, O(log n) searches of a sorted one, O(n) passes.
let n: Int = read()
var t = ""
for i in 0..n: t.push(Char(97 + i * 7919 % 26))
var u = ""
for c in 0..26:
    for i in 0..n / 26: u.push(Char(97 + c))
var s = 0
for i in 0..64 * n:
    let k = i * 13 % n
    t.push('z') // timed: builtin.Str.push
    if let c = t.pop(): s += Int(c) // timed: builtin.Str.pop
    if let c = t.get(k): s += Int(c) // timed: builtin.Str.get
    t.swap(k, (k + 1) % n) // timed: builtin.Str.swap
    s += t.cut_after(k) // timed: builtin.Str.cut_after
    s += t.cut_before(k) // timed: builtin.Str.cut_before
for i in 0..4 * n:
    let c = Char(97 + i % 26)
    s += u.first_ge(c) ?? 0 // timed: builtin.Str.first_ge
    s += u.first_gt(c) ?? 0 // timed: builtin.Str.first_gt
    s += u.last_le(c) ?? 0 // timed: builtin.Str.last_le
    s += u.last_lt(c) ?? 0 // timed: builtin.Str.last_lt
let low = (c: Char) => c >= 'a' // within: builtin.Str.all
let up = (c: Char) => c < 'a' // within: builtin.Str.any
let code = (c: Char) => Int(c) // within: builtin.Str.map
let vowel = (c: Char) => c == 'a' or c == 'e' // within: builtin.Str.filter
let rise = (p: Char, q: Char) => p == q // within: builtin.Str.reach
for r in 0..1024:
    var c = t.clone()
    c.reverse() // timed: builtin.Str.reverse
    s += t.reversed().len // timed: builtin.Str.reversed
    s += t.map(code).len // timed: builtin.Str.map
    s += t.filter(vowel).count() // timed: builtin.Str.filter
    s += t.count('A') // timed: builtin.Str.count
    if let m = t.min(): s += Int(m) // timed: builtin.Str.min
    if let m = t.max(): s += Int(m) // timed: builtin.Str.max
    if t.all(low): s += 1 // timed: builtin.Str.all
    if t.any(up): s += 1 // timed: builtin.Str.any
    s += u.reach(while: rise).len // timed: builtin.Str.reach
for r in 0..32:
    var c = t.clone()
    c.sort() // timed: builtin.Str.sort
    s += t.sorted().len // timed: builtin.Str.sorted
print(s)

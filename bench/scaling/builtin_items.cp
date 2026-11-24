// n: 2048
// `.items` is a view: a read of it runs in the reader's loop and builds no list of pairs, so
// `first` costs the same at 4n. A list built per call reads 4x here, a quadratic 16x; both fail.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let a: [n]Int = [i * 7919 % n for i in 0..n]
var t = ""
for i in 0..n: t.push(Char(97 + i * 7919 % 26))
var s = 0
for i in 0..64 * n:
    s += (xs.items.first ?? (0, 0)).1 // timed: builtin.List.items
    s += (a.items.first ?? (0, 0)).1 // timed: builtin.Array.items
    if let p = t.items.first: s += Int(p.1) // timed: builtin.Str.items
print(s)

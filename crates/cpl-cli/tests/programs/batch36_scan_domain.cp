let n = 4
domain V = 0..n
let a: [V]Int = [3, 1, 4, 1]
let s = scan(a, +)
for v in V: print(s[v])
let b: [1..=3]Int = [5, 6, 7]
let t = scan(b, max)
print(t[3], t)
var e: List<Int> = []
e.push(4)
e.push(5)
var r = scan(e, +)
r.push(1)
print(r)
print(scan([1, 2, 3], +))
print(sum(scan([1, 2, 3], +)))
let w = [2, 3, 4]
print(scan(w, *), scan(w.rev, min))
let none_: List<Int> = []
print(scan(none_, +).len)
let fs: List<F64> = [0.5, 1.5]
print(scan(fs, +))
print(scan([x * x for x in 1..=4], +))
let hs = "hello"
print(scan(hs, max))
print(scan("abca", min).len)

// Two lists grown by push in a program that sorts, as ioi08p1 builds its
// output: each doubling frees the old buffer, and a freed big buffer must go
// back to the OS rather than stay as a hole the next, bigger one can't use.
let n: 1..=10_000_000 = read()
var es = [(i * 7919 % 1000, i) for i in 0..1000]
es.sort(by: e => e.0)
var a: List<Int> = []
var b: List<Int> = []
for i in 0..n:
    a.push(i + es[i % 1000].1)
    b.push(i * 2)
print(a[n - 1] + b[n / 2] + a.len)

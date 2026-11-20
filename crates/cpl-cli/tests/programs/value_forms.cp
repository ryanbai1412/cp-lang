// Library fns that read as methods: `x.bit(i)`, `xs.deduped()`,
// `xs.partition(keep)`, `s.repeat(n)`.
let x = 10
print(x.bit(1), x.bit(0), x.bit(3), x.bit(40))
let d = sorted([3, 1, 3, 2, 1]).deduped()
print(d)
let (small, big) = [5, 1, 8, 2].partition(e => e < 4)
print(small, big)
var s = "0".repeat(3)
print(s, "ab".repeat(2))

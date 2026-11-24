// expect: ok, output "2 0 2"
// `Counter(xs)` counts each element, as Python's `Counter(xs)` does (it used to read `xs` as `init`).
let a: List<Int> = [3, 1, 3]
let c = Counter(a)
print(c[3], c[2], c.len)

// expect: ok, output "2"
// `HashSet(xs)` holds the distinct elements of `xs`, as Python's `set(xs)` does.
let a: List<Int> = [3, 1, 3]
print(HashSet(a).len)

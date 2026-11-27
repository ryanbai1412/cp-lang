// expect: ok, output "1 2 3 3\n2 2\n2 3 4"
// rule: R02.3.3 R04a.2.3
// `xs + s` is a new list: `xs`'s elements, then those `s` walks, whether `s` is a list or a
// range; `xs` is left as it was.
let xs = [1, 2]
let a = xs + [3]
print(a, a.len)
let b = [(0, 0)] + [(1, 2)]
print(b.len, b[1].1)
print([2] + (3..5))

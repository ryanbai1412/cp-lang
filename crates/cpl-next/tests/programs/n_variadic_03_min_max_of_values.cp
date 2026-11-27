// expect: ok, output "4 3\n9 1 6\nb 1 9"
// rule: 11.3.3
// `min` and `max` are ordinary core fns over two or more values of one `Ord` type.
print(max(3, 4), min(3, 4))
print(max(1, 9, 2), min(5, 2, 8, 1), max(1, 2, 3, 4, 5, 6))
let t = min((2, 1), (1, 9))
print(max("b", "a"), t.0, t.1)

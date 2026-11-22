// expect: ok, output "1 a\n1 2 3\n1\n2\n1 -1\n12"
// rule: 10.3.1
print(1, "a")
print([1, 2, 3])
print([1, 2], sep: "\n")
print([Some(1), None], none: -1)
print(1, end: "")
print(2)

// expect: ok, output "same differ"
// engines: next
print(if (1, 2, 3, 4).hash() == (1, 2, 3, 4).hash(): "same" else: "differ", if (1, 2, 3, 4).hash() == (4, 3, 2, 1).hash(): "same" else: "differ")

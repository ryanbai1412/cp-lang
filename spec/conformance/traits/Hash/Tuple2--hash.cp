// expect: ok, output "same differ"
// engines: next
print(if (1, 2).hash() == (1, 2).hash(): "same" else: "differ", if (1, 2).hash() == (2, 1).hash(): "same" else: "differ")

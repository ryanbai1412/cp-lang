// expect: ok, output "same differ"
// engines: next
print(if (1, "a", 'c').hash() == (1, "a", 'c').hash(): "same" else: "differ", if (1, "a", 'c').hash() == (1, "b", 'c').hash(): "same" else: "differ")

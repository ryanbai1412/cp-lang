// expect: ok, output "same differ"
// engines: next
let a = Tuple1(5)
print(if a.hash() == Tuple1(5).hash(): "same" else: "differ", if a.hash() == Tuple1(6).hash(): "same" else: "differ")

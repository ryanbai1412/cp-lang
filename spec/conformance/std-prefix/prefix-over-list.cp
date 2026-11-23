// expect: ok, output "9 5"
// rule: 07.9.1
// `Prefix` takes a `List<Int>` as well as an array, and folds an `Int` cut range (rebuild 7, H-PREFIX-LIST).
let xs = [1, 4, 5, 2]
let p = Prefix(xs)
print(p.fold(1..3), p.fold(0..2))

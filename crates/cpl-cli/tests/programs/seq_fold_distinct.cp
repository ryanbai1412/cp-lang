fn combine(a: Int, b: Int) -> Int = a * 10 + b

let xs = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3]
print(xs.distinct())
print(xs[2..8].distinct())
print(xs.rev.distinct())

let arr: [0..5]Int = [12, 18, 12, 24, 18]
print(arr.distinct())

let s = "abracadabra"
print(s.distinct())
print(s[2..7].distinct())
print((0..4).distinct())

let empty: List<Int> = []
print(empty.distinct())

// Monoid fold (returns identity on empty)
print(arr.fold(gcd))
print(xs[..4].fold(+))
print(empty.fold(+))
print(empty.fold(gcd))

// Binary-fn fold (same empty-sequence rule as min/max: T or ?T with ! / ?? / if let)
let digits = [1, 2, 3, 4]
print(digits.fold((a, b) => a * 10 + b))
print(digits.fold(combine))
print(digits[1..].fold(combine))
let darr: [0..3]Int = [7, 8, 9]
print(darr.fold((a, b) => a * 10 + b))
print((1..=4).fold((a, b) => a * 10 + b))
print(digits.fold((a, b) => a + b)!)
print(empty.fold((a, b) => a + b) ?? -1)
print(empty.fold(combine) ?? -2)

// View min/max with ?? now also returns ?T
let empty_view = empty[0..0]
print(empty_view.min() ?? -3)
print(empty_view.max() ?? -4)
print(empty_view.fold((a, b) => a + b) ?? -5)

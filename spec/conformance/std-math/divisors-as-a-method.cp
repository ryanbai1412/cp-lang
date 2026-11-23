// expect: ok, output "1 2 3 4 6 12\n1 3 9"
// rule: std.math divisors
// `divisors` reads as a method on an `Int` as well as a call.
print(12.divisors())
print(divisors(9))

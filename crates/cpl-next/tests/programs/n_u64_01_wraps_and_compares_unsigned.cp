// expect: ok, output "18446744073709551615 0 1 6 3 YES 9223372036854775807 15755400384260043839"
// rule: R02.1, R03.4.1
// `U64` wraps, compares unsigned, and shifts by an `Int` count.
let m = ~U64(0)
let z = m + U64(1)
print(m, z, U64(3) & U64(5), U64(3) ^ U64(5), U64(12) >> 2, yes(U64(-1) > U64(1)), Int(m >> 1), U64(-7046029254386353131) * U64(3))

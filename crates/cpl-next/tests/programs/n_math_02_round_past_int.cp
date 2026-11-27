// expect: R0202
// An `F64` whose rounding is not an `Int` traps rather than wrapping (R02.11.1).
let big = 10000000000.0 * 1000000000.0
print(floor(big)) // <- here
